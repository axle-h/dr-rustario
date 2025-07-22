use std::collections::HashSet;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::generation_stats::GenerationStatistics;
use crate::game::ai::genome::Genome;
use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
use crate::game::ai::linear::LinearCoefficients;
use crate::game::ai::mutation::{GenomeMutation, RateLimits};
use crate::game::ai::organism::Organism;
use crate::game::ai::record::GenerationRecord;

#[derive(Debug, Clone, Copy)]
pub struct HyperParameters {
    population_size: usize,
    elite_count: usize, // elites are passed onto the next generation unchanged
    survivor_count: usize, // only survivors are selected to breed
    parent_count: usize, // the number of breeding pairs each generation, the parents are selected from the surviving population weighted by fitness
    end_game: EndGame,
    max_generations: usize,
    max_stale_generations_per_seed: usize,
}

impl HyperParameters {
    pub fn new(population_size: usize, elite_rate: f64, survival_rate: f64, end_game: EndGame, max_generations: usize, max_stale_generations_per_seed: usize) -> Self {
        fn rate_to_count(population_size: usize, rate: f64) -> usize {
            assert!(rate >= 0.0 && rate <= 1.0, "rates must be between 0.0 and 1.0");
            (population_size as f64 * rate) as usize
        }

        let elite_count = rate_to_count(population_size, elite_rate);
        let survivor_count = rate_to_count(population_size, survival_rate);

        assert!(elite_count + survivor_count < population_size, "too many elites and survivors");

        Self {
            population_size,
            elite_count,
            survivor_count: rate_to_count(population_size, survival_rate),
            parent_count: ((population_size as f64 - elite_count as f64) / 2.0).ceil() as usize,
            end_game,
            max_generations,
            max_stale_generations_per_seed
        }
    }
}

impl Default for HyperParameters {
    fn default() -> Self {
        Self::new(
            1000,
            0.005,
            0.5,
            EndGame::NONE,
            usize::MAX,
            10
        )
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
    action_evaluator_factory: F,
}

impl<const N: usize, F> GeneticAlgorithm<N, F>
where F : Fn(&Genome<N>) -> ActionEvaluator + Send + Sync
{
    pub fn new(
        fixture: HeadlessGameFixture,
        mut mutation: GenomeMutation<N>,
        hyper_parameters: HyperParameters,
        population_seed: Option<Genome<N>>,
        action_evaluator_fn: F
    ) -> Self {
        let genome_seed: Option<Genome<N>> = population_seed.map(|seed| seed.into());
        let mut population = Vec::with_capacity(hyper_parameters.population_size);
        for _ in 0 .. hyper_parameters.population_size {
            let genome = if let Some(genome_seed) = genome_seed {
                mutation.mutate(genome_seed)
            } else {
                mutation.random()
            };
            population.push(Organism::new(genome));
        }

        Self {
            population,
            generations: vec![],
            fixture,
            mutation,
            hyper_parameters,
            action_evaluator_factory: action_evaluator_fn
        }
    }

    pub fn run(&mut self) -> GenerationStatistics<N> {
        println!("Running genetic algorithm...");

        let mut record = GenerationRecord::new().expect("Failed to create generation record");
        println!("Results saved to {}", record.path().display());

        let t0 = Instant::now();
        loop {
            let stats = self.evolve();
            println!("{}", stats);
            record.add(&stats).expect("Failed to write to generation record");
            
            let id = stats.id();
            if id >= self.hyper_parameters.max_generations ||
                self.hyper_parameters.end_game.is_end_game(stats.max().result(), Instant::now() - t0) {
                return stats
            }

            if self.generations.len() < self.hyper_parameters.max_stale_generations_per_seed {
                continue; // not enough generations to check for staleness
            }

            let recent_scores = self.generations.iter()
                .rev()
                .take(self.hyper_parameters.max_stale_generations_per_seed)
                .map(|s| s.max().result().score())
                .collect::<HashSet<_>>();

            if recent_scores.len() == 1 {
                self.fixture.next_seed();
                self.population.iter_mut().for_each(Organism::unset_result);
                println!("score is stale, using new seed {}", self.fixture.current_seed())
            }
        }
    }
    
    fn evolve(&mut self) -> GenerationStatistics<N> {
        // Calculate fitness in parallel
        let generation_start = Instant::now();
        self.population
            .par_iter_mut()
            .for_each(|member| {
                member.set_result(|genome| self.fixture.play((self.action_evaluator_factory)(genome)));
            });
        self.population.sort_by(|s1, s2| s2.result().cmp(&s1.result()));
        let generation_duration = generation_start.elapsed();

        // Calculate total gameplay time
        let total_gameplay_time: Duration = self.population.iter()
            .map(|organism| organism.result().time())
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
        self.mutation.add_sample(stats);

        self.next_generation();

        stats
    }

    fn next_generation(&mut self) {
        let surviving_population: Vec<_> = self.population.iter()
            .take(self.hyper_parameters.survivor_count)
            .copied()
            .collect();

        self.population.clear();

        for elite in surviving_population.iter().take(self.hyper_parameters.elite_count) {
            // includes cached result
            self.population.push(*elite);
        }

        let parents = self.mutation.parents(&surviving_population, self.hyper_parameters.parent_count);

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

pub fn ga_main_linear() -> Result<(), String> {
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        rand::random(),
        HeadlessGameOptions::default(),
        EndGame::of_lines(10_000) // TODO what is the world record?
    );
    let mutation = GenomeMutation::of_max(
        RateLimits::new(0.1 ..= 0.20),
        RateLimits::new(0.1 ..= 0.20),
        5,
        rand::random()
    );
    GeneticAlgorithm::new(
        fixture,
        mutation,
        HyperParameters::default(),
        None,
        move |&genome| ActionEvaluator::Linear(genome.into())
    ).run();
    
    Ok(())
}

pub fn ga_main_neural() -> Result<(), String> {
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        rand::random(),
        HeadlessGameOptions::default(),
        EndGame::of_lines(10_000) // TODO what is the world record?
    );
    let mutation = GenomeMutation::of_max(
        RateLimits::new(0.1 ..= 0.20),
        RateLimits::new(0.1 ..= 0.20),
        5,
        rand::random()
    );
    GeneticAlgorithm::new(
        fixture,
        mutation,
        HyperParameters::default(),
        None, //Some(TetrisNeuralNetwork::default().into()),
        move |&genome| ActionEvaluator::NeuralNetwork(genome.into())
    ).run();

    Ok(())
}

fn deterministic() {
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        "39447712892375752940097570078805324248880289121439932390588186706160814925155"
            .to_string()
            .into(),
        HeadlessGameOptions::default(),
        EndGame::of_lines(100_000)
    );;
    // let evaluator = ActionEvaluator::NeuralNetwork(TetrisNeuralNetwork::new(
    //     &[-308240, 281286, -620154, -588640, 97170, 554897, -815713, -1071458, 707333, -330713, -856277, -507184, -448677, -257718, 402380, 909949, 789321, -499823, -773522, -548976, -191959, 174290, -745140, -924392, -369651, 300787, -1040824, 3635, 833073, -419689, 683369, -61446, -65944, -794899, 148463, 654495, -944143, -280775, 951878, -886058, 179280, 750696, -432313, 475014, -240554, -199543, 693950, 536456, -919240, 689967, -509990, 830931, -214666, 373079, 927748, -99831, -572605, 625395, -84074, -444866, 753329, -833524, -356895, 41325, -330982, 880029, 729862, 775150, -887386, 425332, -996234, 565430, 252925, 984793, 742993, 673686, 821585, -224610, -43350, 831459, -121761, 641595, -633126, -338161, 736187, 486206, -598947, 267029, -457479, 956695, -115825, 750221, 78758, 810384, -475951, 833158, -3788, 9644, -425043, 152708, 888564, -586121, 106840, 540701, 76394, -170722, -230706, 1001827, -33667, -907825, -404291, 1056844, 222462, -928792, 645387, 76348, -883429, -384574, -519000, 141027, -1056844, 237706, 476943, 363517, 177554, 418841, -222533, -93338, -60594, 40815, 798874, -255698, -521608, 861850, 996363, 349554, -893839, -85462, 771364, 711376, -165989, -450096, 466041, -944100, 824812, 894826, -90040, -804106, 526804, -549328, -937127, 67423, 22266, 784342, -421846, -606182, -127271, -193398, 517694, -845791, 897186, -922279, -878294, 671023, 479633, 532940, 117603, -980810, -990874, -638217, 790835, 643705, 508566, 648244, -638035, -358876, -422736, -713681, -99077, -644295, 811251, 895308, 95054, 781120, 995103, 462205, 786850, -477124, -210991, -757341, -631707, -631277, -294076, 38157, 762488, 410578, 139114, 348527, -235612, 750200, 960776, -10249, 182460, 986821, 509096, 983227, -623227, 230273, -684623, 578499, 526572, 578016, -224463, 528273, -548852, 414013, 56186, 46565, 284184, 343274, -506894, -1158643, 466006, -864725, -53175, -136160, 792660, 190870, -819603, 558053, 220894, -446201, -936967, -938909, -661461, -481571, -632487, 244108, -896044, 559176, 1022784, -356316, 176521, 561787, 177058, -334471, -779374, 826144, -403137, 373446, -345351, -682244, 905742, 421154, -426956, -260354, -612842, -111929, 660659, 364070, -462090, 49670, -709947, 408012, 771692, 618507, -110459, -118106, 662046, -966900, 180323, -904554, -67600, -306641, -637837, -177002, -685180, -674919, 646246, -933348, -600445, 191828, -394218, -1036970, 336100, -394951, -10374, 546824, 752947, 277411, -894123, 625040, -789031, -398509, 261903, -215925, -14415, 914635, -228533, 899521, -401767, 225987, -480878, -783871, -551231, -174414, 237972, -844153, 401322, -319874, 659264, 442904, -935395, 571904, -254084, -364640, 847223, -726889, -943591, 73871, -22084, -320947, -101189, -403401, 322101, -52466, -204212, -939059, 712159, 770472, -511368, -573585, 611320, -571071, 365003, -858469, 18535, 111476, -975284, -467192, -583475, 832795, 163474, 933548, -895633, 161666, -341172, 1030507, 241532, -70616, -500577, -568777, 945459, 68910, -434465, -427457, -558553, -596226, 791935, 326898, 626596, 533213, 257125, -748982, 888702, 995911, -181318, 897698, 193604, -179585, 504114, -543528, -749418, 466156, 874855, -988162, -143667, -53111, 104651, 820393, -175763, 340195, 166408, 700207, 26570, 765611, -697982, -684559, 144040, -824323, 635938, -906859, -311143, -561811, -31480, 733145, 453674, -839241, -512046, -457807, -728576, 855820, 402401, -209680, -775858, -826810, 449635, 623980, -148641, -205989, -959110, -306444, 179497, 167992, -507966, 429063, -307001, -180369, -545841, 251177, -587728, -376026, 755559, 585325, 667920, -211494, -469618, 51979, 659176, -312345, -943299, -135039, -1120257, 437980, 296689, 856544, 578975, 143263, 489506, -52274, 256088, -598544, 288477, -142016, 648315, -448533, 463099, 607577, -569655, 129305, -129527, -833640, 311573, 41965, 277064, 454161, 761510, -722587, -575179, 758327, 978159, 543530, 220724, 346726, 367575, 364972, -744738, 495480, -799990, 385004, 142260, 522375, 747593, -725307, 140700, 860032, 690048, 495727, 382968, 173737, 161359, -814462, 740388, -667896, 410731, 704830, 717931, -515631, -756313, 159770, 1044187, -507436, -411520, 143100, -534462, 108942, 480163, -850560, -436142, 280593, 782389, -676926, 265698, -579949, 734707, 644881, 338861, -251657, 639564, -595062, -487767, -221055, 155322, 678427, 26736, -812675, -669157, -214155, 404774, 199140, 283279, 797268, -959213, 678191, 132530, -172851, 83229, -55227, 198079, -578040, 637706, 584511, -52358, -336657, 418832, -721621, 272581, 409879, -732927, -328185, -831542, 405983, 796098, 584003, -472300, 195519, 907893, -813763, -134781, 551029, 218435, 857362, 266322, -233261, -167306, -923254, -910166, 400312, -97627, 44506, -973042, -534961, -766416, -1063029, -1062645, 549461, 577602, -877112, 11424, 14098, -675325, 956359, -560578, 372003, -400342, -110014, 113871, -444900, -662436, -799539, -1056595, -936222, -150349, -820266, 262126, -307503, -72683, 364672, -96976, -523050, 80114, -1003515, 878825, -772825, 31370, -736495, -1066210, -126025, 982383, 959825, -487678, -261717, 214350, -60413, -209700, -136080, 676121, -127811, -861796, -850245, 700900, 254609, 64921, 189656, -941301, -344965, 357355, -543386, -907484, -63693, 530798, 264229, -592576, -373325, -432914, -129326, -350688, 681254, 568403, -133059, -133098, 847478, 983003, 69197, -521456, -220039, 153068, -172860, -417241, 489882, -22765, -3127, 815954, 258961, 362768, -226428, -227536, -110803, 245063, -168600, -165616, 420773, -79713, 543775, -636898, -625929, 11908, -54044, -352811, 411765, 7939, 388885, -90285, -772298, 344105, 550804, 1041833, 555577, -261771, 308206, 111546, 221072, 882294, 302589, -405672, -180932, 852629, 134564, 345926, -904740, -635551, 481415, -500854, 854454, 30086, 694799, -498807, 868742, 591249, 517125, -911626, 207407, 605356, -341520, -600701, 1033005, 581855, 64374, -394472, -472378, -189140, -136989, 423611, 442430, 737258, 807562, 943577, -190211, 80274, 283973, -905701, -1006501, 401098, 694790, -705208, 726854, -41408, 394749, 730809, 835328, 364865, 309971, 627768, -4067, -616452, -707250, -619702, -537774, 464523, -31292, -772058, -486626, 847928, -342819, 198077, 678521, 806403, -247153, -368057, -607497, -596454, 608162, -994876, 800436, -873564, 545129, 782537, -108360, 762504, 508343, -855950, 807349, -762269, 663638, 316163, 149085, 294927, 146155, -797603, -51867, -811516, -522359, -951917, -592130, 45784, 653868, 756331, 768212, -199973, -757680, -545326, -602054, 505953, -461830, -539944, 683663, 428939, -83516, 404854, 414182, 360111, 684105, -266779, -722493, -913055, 129608, 811893, 984432, -838848, -201572, -966683, 464913, -43867, 164210, 728411, 980190, 595678, -846224, 564984, 800780, -424379, -44050, -828512, 952578, -148547, -922434, -596433, -61455, -62197, -827473, -223030, 324199, -629993, 189387, 651584, 807126, 415683, 859791, 498215, -745243, 452011, 287247, 58683, -243569, -912930, 388093, -387007, -54007, -847735, -734509, 898265, 758799, -508690, 271616, 346841, 566998, 896164, 951983, -675352, 89652, -513988, -1014590, 406068, 851758, 62488, -855960, 104877, 812970, 659284, 821680, 491186, 166401, 453173, 195193, -647197, -447826, -315988, 633562, 827900, -372878, 152519, -1030981, 412752, -319275, 224155, 1034175, -838976, 206367, -720095, -457437, -117079, 358206, -744717, -839267, -988202, 347473, 432365, -1021887, -235891, 850997, -473921, 988441, -22760, -189478, 471274, -285542, 784123, 585196, -625549, 247799, -651925, 451091, 914813, 962255, -931747, -670311, -500879, 814657, -766199, 493371, -166323, 340498, 630984, -280463, -251168, 390393, 1058173, -197828, 250757, 4483, 185476, 797987, -1125798, -181926, -579421, 54810, 690744, -833082, 76930, 527758, -351952, -766190, -93276, -898975, -634722, 531169, 633716, 147550, 166778, 926885, 269204, -824047, -1058847, 964833, -173842, 439990, 94705, -340005, -348514, -398719, -212204, -357257, 579871, -771854, -760511, 422747, 508986, -315105, -395165, -277142, 102110, 1082883, 394078, 906778, 617728, -462933, -490180, 296807, 143822, 341284, -548199, 931053, 561116, -615458, -336089, 188397, 601465, 52675, 105745, 775222, 185320, 329343, 659746, 709175, 368940, -592012, 966211, -370672, 359503, -586815, 539710, 969633, -552026, 872687, -244939, -444717, 655224, -822298, -300259, -143829, 51654, 533638, -402098, 640490, 1142333, 542957, 1078235, 995508, -477236, 406795, 375853, -375122, 222777, 756400, 930103, 450280, -534694, -49816, -898398, -920939, -808015, 746957, 352656, -894986, 124715, 789440, 695642, -586912, 790629, -199475, 869315, 865332, 897115, -96161, -284252, -283493, -983065, -157207, 587829, 841558, -577051, -779135, -934052, -641710, 965730, 108564, -573490, 6519, -312229, 191236, 981381, 942510, -891660, 589229, -539716, -382221, 481258, -313700, -406070, 517031, 837924, 426631, -760064, -1007862, 230911, 796012, 905394, -959376, -187455, -695043, -29412, 807555, -718419, 476284, -496418, -179600, -836311, -642215, 486385, 810931, -181839, 658374, -781953, 777761, -268178, -268420, 1021994, -166660, -526567, 708343, -1011397, -133990, -834434, -311299, 66776, -463062, 814659, -42077, -796601, -436568, -321976, -201116, 361571, 221197, 866759, -336425, -852874, 735489, 800565, -453778, 236016, -780445, 768584, 79575, -572800, 299692, 81583, 157660, -278605, -492474, -54020, 533835, -908457, 278703, 306643, -199224, 67442, 131937, -771624, 137533, 919272, 688346, 849739, -58053, -461687, 754561, -658018, 859466, 414583, 203396, 476844, 548002, -526283, -308502, -400841, 171394, 608683, 609944, -396971, -605054, 90141, -162937, -513653, 659466, -318209, 139622, 155412, -604687, -258790, 939577, -32590, 32032, -821522, 977998, 351879, -463493, -60481, -840722, 271002, -139444, -644029, -1103963, 451867, -427188, -626072, 824911, 372517, -141392, 92195, -151368, 764964, -178331, -795444, -867593, -524919, 107699, -538467, 319583, -497028, -275152, -410467, -829244, 453545, -62218, 171222, 760823, -175582, -697756, -487394, 601655, -328584, 103678, 424473, 1071043, 635604, 563866, -282209, 790207, -723576, 209128, -845416, 952893, 644418, -883609, -562151, 579192, 16630, 976235, 474298, -215442, -29791, -721846, -580012, -17934, 212631, 816354, 945986, 494500, -867682, 1079715, -137362, -894829, 529989, 422307, 155410, 810288, 597354, -672059, 283251, 285652, -805707, -231485, 370235, -522714, 70104, -418416, 1013234, -608382, 271474, 669600, -370536, 178125, -258492, -305440, 292012, 404683, -361205, 249042, -683746, 715554, -394846, -116119, 975111, -876606, 73454, -645270, 492254, -872012, 467188, 878528, -613785, 425773, 411979, 562873, 110744, 115280, -629194, -881176, 903972, -397889, 817407, -390537, -861117, -906292, 412623, 887308, 155460, 281457, -279729, 963755, 676180]
    //         .map(|c| Coefficient::new(c).into_f64())
    // ));
    // let evaluator = ActionEvaluator::NeuralNetwork(TetrisNeuralNetwork::default());
    let evaluator = ActionEvaluator::Linear(LinearCoefficients::default());

    let result1 = fixture.play(evaluator);
    println!("{}", result1);
    loop {
        let result2 = fixture.play(evaluator);
        println!("{}", result2);
        assert_eq!(result1, result2);
    }

    let result1 = fixture.play(evaluator);
    println!("{}", result1);
    assert_eq!(result1.score(), 3007);

    let result2 = fixture.play(evaluator);
    assert_eq!(result1, result2);
}

pub fn ga_main() -> Result<(), String> {
    deterministic();
    Ok(())
    // ga_main_neural()
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::game::ai::action_evaluator::ActionEvaluator;
    use crate::game::ai::genetic::{GeneticAlgorithm, HyperParameters};
    use crate::game::ai::genome::LinearGenome;
    use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
    use crate::game::ai::mutation::{GenomeMutation, RateLimits};

    #[test]
    fn genetic_algorithm() {
        let fixture = HeadlessGameFixture::new(
            Config::default(),
            100.into(),
            HeadlessGameOptions::default(),
            EndGame::of_seconds(2)
        );
        let mutation = GenomeMutation::of_max(RateLimits::default(), RateLimits::default(), 5, 100.into());
        GeneticAlgorithm::new(
            fixture,
            mutation,
            HyperParameters::new(
                10,
                0.01,
                0.5,
                EndGame::NONE,
                1,
                100
            ),
            None,
            move |&genome| ActionEvaluator::Linear(genome.into())
        ).run();

        assert!(true);
    }
}