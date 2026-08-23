use std::time::{Duration, Instant};
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::game_result::GameResult;
use crate::game::ai::generation_stats::GenerationStatistics;
use crate::game::ai::genome::Genome;
use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
use crate::game::ai::mutation::GenomeMutation;
use crate::game::ai::neural::{NeuralGenome, TetrisNeuralNetwork, NEURAL_GENOME_SIZE};
use crate::game::ai::objective::{Objective, Phase};
use crate::game::ai::organism::Organism;
use crate::game::ai::generation_record::GenerationRecord;

pub const DEFAULT_LINE_CAP: u32 = 10_000;
pub const DEFAULT_PIECE_CAP: u32 = 1_000;

#[derive(Debug, Clone, Copy)]
pub struct HyperParameters {
    population_size: usize,
    elite_count: usize, // elites are passed onto the next generation unchanged
    survivor_count: usize, // only survivors are selected to breed
    parent_count: usize, // the number of breeding pairs each generation, the parents are selected from the surviving population weighted by fitness
}

impl HyperParameters {
    pub fn new(population_size: usize, elite_rate: f64, survival_rate: f64) -> Self {
        fn rate_to_count(population_size: usize, rate: f64) -> usize {
            assert!(rate >= 0.0 && rate <= 1.0, "rates must be between 0.0 and 1.0");
            (population_size as f64 * rate) as usize
        }

        let elite_count = rate_to_count(population_size, elite_rate);
        let survivor_count = rate_to_count(population_size, survival_rate);

        assert!(elite_count + survivor_count < population_size, "too many elites and survivors");
        assert!(survivor_count >= 2, "need at least two survivors to breed");

        Self {
            population_size,
            elite_count,
            survivor_count,
            parent_count: ((population_size as f64 - elite_count as f64) / 2.0).ceil() as usize,
        }
    }
}

impl Default for HyperParameters {
    fn default() -> Self {
        Self::new(1000, 0.005, 0.5)
    }
}

pub struct GeneticAlgorithm<const GENOME: usize, F>
where F : Fn(&Genome<GENOME>) -> ActionEvaluator
{
    population: Vec<Organism<GENOME>>,
    generations: Vec<GenerationStatistics<GENOME>>,
    fixture: HeadlessGameFixture,
    mutation: GenomeMutation<GENOME>,
    hyper_parameters: HyperParameters,
    phases: Vec<Phase>,
    phase_index: usize,
    phase_generations: usize,
    action_evaluator_factory: F,
}

impl<const N: usize, F> GeneticAlgorithm<N, F>
where F : Fn(&Genome<N>) -> ActionEvaluator + Send + Sync
{
    /// `phases` are run in order; a phase ends when it is complete (see [Phase::is_complete]) or has run
    /// for its `max_generations`, the best member is then used to seed the population of the next phase.
    pub fn new(
        mut fixture: HeadlessGameFixture,
        mut mutation: GenomeMutation<N>,
        hyper_parameters: HyperParameters,
        phases: Vec<Phase>,
        population_seed: Option<Genome<N>>,
        action_evaluator_fn: F
    ) -> Self {
        assert!(!phases.is_empty(), "at least one phase is required");
        Self::apply_phase(&phases[0], &mut fixture, &mut mutation);

        let population = Self::initial_population(&hyper_parameters, &mut mutation, population_seed);

        Self {
            population,
            generations: vec![],
            fixture,
            mutation,
            hyper_parameters,
            phases,
            phase_index: 0,
            phase_generations: 0,
            action_evaluator_factory: action_evaluator_fn
        }
    }

    fn apply_phase(phase: &Phase, fixture: &mut HeadlessGameFixture, mutation: &mut GenomeMutation<N>) {
        fixture.set_end_game(phase.end_game);
        fixture.set_seeds_per_game(phase.seeds_per_game);
        mutation.set_rates(phase.mutation_rate.clone(), phase.crossover_rate.clone(), phase.mutation_step);
    }

    /// a seeded population keeps one pristine copy of the seed, the rest are mutations of it
    fn initial_population(
        hyper_parameters: &HyperParameters,
        mutation: &mut GenomeMutation<N>,
        population_seed: Option<Genome<N>>
    ) -> Vec<Organism<N>> {
        let mut population = Vec::with_capacity(hyper_parameters.population_size);
        for i in 0 .. hyper_parameters.population_size {
            let genome = match population_seed {
                Some(seed) if i == 0 => seed,
                Some(seed) => mutation.mutate(seed),
                None => mutation.random()
            };
            population.push(Organism::new(genome));
        }
        population
    }

    pub fn phase(&self) -> &Phase {
        &self.phases[self.phase_index]
    }

    pub fn objective(&self) -> Objective {
        self.phase().objective
    }

    pub fn population(&self) -> &[Organism<N>] {
        &self.population
    }

    pub fn run(&mut self) -> GenerationStatistics<N> {
        println!("Running genetic algorithm ({} phase)...", self.objective());

        let mut record = GenerationRecord::new().expect("Failed to create generation record");
        println!("Results saved to {}", record.path().display());

        loop {
            let stats = self.evolve();
            println!("{}", stats);
            record.add(&stats).expect("Failed to write to generation record");

            let phase_over = self.phase().is_complete(&stats.max().result())
                || self.phase_generations >= self.phase().max_generations;
            if !phase_over {
                self.next_generation();
                continue;
            }

            if self.phase_index + 1 >= self.phases.len() {
                return stats;
            }

            self.next_phase(stats.max().genome());
        }
    }

    /// switch to the next phase, re-seeding the population from `best`
    fn next_phase(&mut self, best: Genome<N>) {
        self.phase_index += 1;
        self.phase_generations = 0;
        let phase = self.phases[self.phase_index].clone();
        println!("{} phase complete after generation {}, switching to {} phase", 
                 self.phases[self.phase_index - 1].objective, self.generations.len(), phase.objective);
        Self::apply_phase(&phase, &mut self.fixture, &mut self.mutation);
        self.population = Self::initial_population(&self.hyper_parameters, &mut self.mutation, Some(best));
    }

    /// evaluate the population on fresh seeds and sort it best first, but do not breed
    fn evolve(&mut self) -> GenerationStatistics<N> {
        let objective = self.objective();

        // every generation plays new piece sequences, elites included, so nothing can overfit one seed
        self.fixture.next_seed();
        self.population.iter_mut().for_each(Organism::unset_result);

        // Calculate fitness in parallel
        let generation_start = Instant::now();
        self.population
            .par_iter_mut()
            .for_each(|member| {
                member.set_result(|genome| self.fixture.play((self.action_evaluator_factory)(genome)));
            });
        self.population.sort_by(|s1, s2| objective.cmp(&s2.result(), &s1.result()));
        let generation_duration = generation_start.elapsed();

        // Calculate total gameplay time
        let total_gameplay_time: Duration = self.population.iter()
            .map(|organism| organism.result().time() * self.fixture.seeds_per_game() as u32)
            .sum();

        // Calculate game seconds per real second
        let game_seconds_per_second = if generation_duration.as_secs_f64() > 0.0 {
            total_gameplay_time.as_secs_f64() / generation_duration.as_secs_f64()
        } else {
            0.0 // Avoid division by zero
        };

        let p95_index = (self.hyper_parameters.population_size as f64 * 0.05).floor() as usize;
        let p50_index = self.hyper_parameters.population_size / 2;
        let stats = GenerationStatistics::new(
            self.generations.len() + 1,
            objective,
            self.fixture.current_seed(),
            self.population[0],
            self.population[p95_index],
            self.population[p50_index],
            self.mutation.current_mutation_rate(),
            self.mutation.current_crossover_rate(),
            total_gameplay_time,
            generation_duration,
            game_seconds_per_second
        );
        self.generations.push(stats);
        self.phase_generations += 1;
        self.mutation.add_sample(stats);

        stats
    }

    fn next_generation(&mut self) {
        let objective = self.objective();
        let surviving_population: Vec<_> = self.population.iter()
            .take(self.hyper_parameters.survivor_count)
            .copied()
            .collect();

        self.population.clear();

        for elite in surviving_population.iter().take(self.hyper_parameters.elite_count) {
            self.population.push(*elite);
        }

        let parents = self.mutation.parents(&surviving_population, self.hyper_parameters.parent_count, objective);

        let mut required_children = self.hyper_parameters.population_size - self.population.len();
        while required_children > 0 {
            for [parent1, parent2] in parents.iter() {
                let [child1, child2] = self.mutation.crossover(*parent1, *parent2)
                    .map(Organism::new);
                self.population.push(child1);
                required_children -= 1;

                if required_children > 0 {
                    self.population.push(child2);
                    required_children -= 1;
                }

                if required_children == 0 {
                    break;
                }
            }
        }
    }
}

fn neural_fixture() -> HeadlessGameFixture {
    HeadlessGameFixture::new(
        Config::default(),
        rand::random(),
        HeadlessGameOptions::default(),
        EndGame::NONE
    )
}

fn neural_mutation() -> GenomeMutation<NEURAL_GENOME_SIZE> {
    let phase = Phase::survival(DEFAULT_LINE_CAP);
    GenomeMutation::of_max(phase.mutation_rate, phase.crossover_rate, 5, rand::random())
}

fn run_neural(phases: Vec<Phase>, population_seed: Option<NeuralGenome>) {
    GeneticAlgorithm::new(
        neural_fixture(),
        neural_mutation(),
        HyperParameters::default(),
        phases,
        population_seed,
        move |&genome| ActionEvaluator::NeuralNetwork(genome.into())
    ).run();
}

/// train a random population to survive the line cap
pub fn ga_main_survival() -> Result<(), String> {
    run_neural(vec![Phase::survival(DEFAULT_LINE_CAP)], None);
    Ok(())
}

/// fine tune the built in model for tetris clears within the piece cap
pub fn ga_main_score() -> Result<(), String> {
    run_neural(vec![Phase::score(DEFAULT_PIECE_CAP)], Some(TetrisNeuralNetwork::default().into()));
    Ok(())
}

/// train for survival, then once any member reaches the line cap switch to optimising for tetris clears
pub fn ga_main_auto() -> Result<(), String> {
    run_neural(vec![Phase::survival(DEFAULT_LINE_CAP), Phase::score(DEFAULT_PIECE_CAP)], None);
    Ok(())
}

/// play the built in model on a few seeds under the score phase rules and report how it does
pub fn ga_diagnose() -> Result<(), String> {
    const SEEDS: usize = 4;
    let phase = Phase::score(DEFAULT_PIECE_CAP);
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        1.into(),
        HeadlessGameOptions::default(),
        phase.end_game
    );
    let evaluator = ActionEvaluator::NeuralNetwork(TetrisNeuralNetwork::default());
    println!("built in neural network, {} pieces per game", DEFAULT_PIECE_CAP);

    let results: Vec<_> = (0 .. SEEDS as u128)
        .into_par_iter()
        .map(|seed| (seed, fixture.play_seed(evaluator, (seed + 1).into())))
        .collect();

    for (seed, result) in results.iter() {
        println!("seed {}: {} tetris fraction: {:.3}", seed + 1, result, result.tetris_fraction());
    }
    let mean: GameResult = results.iter().map(|(_, r)| *r).sum::<GameResult>() / SEEDS;
    println!("mean: {} tetris fraction: {:.3} fitness ({}): {}", mean, mean.tetris_fraction(), phase.objective, phase.objective.fitness(&mean));
    Ok(())
}

pub fn ga_main() -> Result<(), String> {
    ga_main_auto()
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::game::ai::action_evaluator::ActionEvaluator;
    use crate::game::ai::genetic::{GeneticAlgorithm, HyperParameters};
    use crate::game::ai::genome::{LinearGenome, LINEAR_GENOME_SIZE};
    use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
    use crate::game::ai::linear::LinearCoefficients;
    use crate::game::ai::mutation::{GenomeMutation, RateLimits};
    use crate::game::ai::objective::{Objective, Phase};

    fn fixture() -> HeadlessGameFixture {
        HeadlessGameFixture::new(
            Config::default(),
            100.into(),
            HeadlessGameOptions::default(),
            EndGame::NONE
        )
    }

    fn mutation() -> GenomeMutation<LINEAR_GENOME_SIZE> {
        GenomeMutation::of_max(RateLimits::default(), RateLimits::default(), 5, 100.into())
    }

    #[test]
    fn genetic_algorithm() {
        let phase = Phase::survival(5).with_max_generations(1);
        GeneticAlgorithm::new(
            fixture(),
            mutation(),
            HyperParameters::new(10, 0.01, 0.5),
            vec![phase],
            None,
            move |&genome| ActionEvaluator::Linear(genome.into())
        ).run();
    }

    #[test]
    fn seeded_population_keeps_a_pristine_seed() {
        let seed: LinearGenome = LinearCoefficients::default().into();
        let mut phase = Phase::score(50).with_max_generations(1);
        phase.seeds_per_game = 2;
        let ga = GeneticAlgorithm::new(
            fixture(),
            mutation(),
            HyperParameters::new(10, 0.01, 0.5),
            vec![phase],
            Some(seed),
            move |&genome| ActionEvaluator::Linear(genome.into())
        );
        assert_eq!(ga.population()[0].genome(), seed);
        assert!(ga.population().iter().skip(1).any(|o| o.genome() != seed));
    }

    #[test]
    fn switches_from_survival_to_score_at_the_line_cap() {
        let seed: LinearGenome = LinearCoefficients::default().into();
        let mut ga = GeneticAlgorithm::new(
            fixture(),
            mutation(),
            HyperParameters::new(10, 0.01, 0.5),
            vec![Phase::survival(5), Phase::score(50).with_max_generations(1)],
            Some(seed),
            move |&genome| ActionEvaluator::Linear(genome.into())
        );
        assert_eq!(ga.objective(), Objective::Survival);
        let stats = ga.run();
        assert_eq!(ga.objective(), Objective::Score);
        assert_eq!(stats.objective(), Objective::Score);
        assert_eq!(ga.fixture.end_game().pieces, 50);
        assert_eq!(ga.fixture.seeds_per_game(), 4);
        assert_eq!(stats.max().result().pieces(), 50);
        // the survival phase should finish in one generation since the default coefficients survive 5 lines
        assert_eq!(stats.id(), 2);
    }
}
