use std::array::from_fn;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign};
use rand::distr::{Distribution, StandardUniform};
use rand::{Rng, RngExt};
use crate::game::ai::coefficient::Coefficient;
use crate::game::ai::genome::Genome;

#[derive(Copy, Clone, PartialEq)]
pub struct Tensor<const R: usize, const C: usize = 1> {
    data: [[f64; C]; R],
}

impl<const R: usize, const C: usize> Tensor<R, C> {

    pub fn rows(&self) -> usize {
        R
    }

    pub fn cols(&self) -> usize {
        C
    }


    const ZEROS: Self = Self {
        data: [[0.0; C]; R],
    };

    const ONES: Self = Self {
        data: [[1.0; C]; R],
    };

    pub fn new(data: [[f64; C]; R]) -> Self {
        Self { data }
    }

    pub const TOTAL_SIZE: usize = R * C;

    pub fn from_slice(data: &[f64]) -> Self {
        debug_assert_eq!(data.len(), Self::TOTAL_SIZE,
           "Invalid data length for Tensor<{}, {}>: expected {}, got {}",
           R, C, Self::TOTAL_SIZE, data.len()
        );
        let mut result = Self::ZEROS;
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = data[i * C + j];
            }
        }
        result
    }

    pub fn flatten(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(Self::TOTAL_SIZE);
        for row in self.data.iter() {
            for col in row.iter() {
                result.push(*col);
            }
        }
        result
    }
    
    pub fn dot<const R2: usize, const C2: usize>(&self, other: &Tensor<R2, C2>) -> Tensor<R, C2> {
        debug_assert_eq!(C, R2, "Cannot multiply tensors with incompatible dimensions");

        let mut result = Tensor::ZEROS;

        for i in 0..self.rows() {
            for j in 0..other.cols() {
                for k in 0..self.cols() {
                    result.data[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }

        result
    }
    fn relu_mut(&mut self) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.data[i][j] = relu(self.data[i][j]);
            }
        }
    }

    fn sigmoid_mut(&mut self) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.data[i][j] = sigmoid(self.data[i][j]);
            }
        }
    }

    fn mcculloch_pitts_mut(&mut self, threshold: f64) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.data[i][j] = mcculloch_pitts(self.data[i][j], threshold);
            }
        }
    }

    fn fmt(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
        let mut formatted_nums = Vec::with_capacity(R * C);
        let mut col_widths = vec![0; C];
        for row in self.data.iter() {
            for (col_idx, val) in row.iter().enumerate() {
                let formatted = format!("{:.6}", val);
                col_widths[col_idx] = col_widths[col_idx].max(formatted.len());
                formatted_nums.push(formatted);
            }
        }

        let indent_str = " ".repeat(indent);
        for i in 0..R {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}[", indent_str)?;
            for j in 0..C {
                if j > 0 {
                    write!(f, " ")?;
                }
                let num = &formatted_nums[i * C + j];
                write!(f, "{:>width$}", num, width = col_widths[j])?;
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}


fn relu(x: f64) -> f64 {
    x.max(0.0)
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn mcculloch_pitts(x: f64, threshold: f64) -> f64 {
    if x > threshold { 1.0 } else { 0.0 }
}

fn activate(x: f64, activation: ActivationFunction) -> f64 {
    match activation {
        ActivationFunction::Identity => x,
        ActivationFunction::ReLU => relu(x),
        ActivationFunction::Sigmoid => sigmoid(x),
        ActivationFunction::McCullochPitt(threshold) => mcculloch_pitts(x, threshold),
    }
}

impl<const SIZE: usize> Tensor<SIZE> {
    pub fn vector(data: [f64; SIZE]) -> Self {
        let mut result = Self::ZEROS;
        for i in 0..SIZE {
            result.data[i][0] = data[i]
        }
        result
    }

    pub fn into_diagonal(self) -> Tensor<SIZE, SIZE> {
        let mut result = Tensor::ZEROS;
        for i in 0..SIZE {
            result.data[i][i] = self.data[i][0]
        }
        result
    }

    fn activate_mut(&mut self, activation: [ActivationFunction; SIZE]) {
        for i in 0..SIZE {
            self.data[i][0] = activate(self.data[i][0], activation[i])
        }
    }
}

impl<const SIZE: usize> Tensor<SIZE, SIZE> {
    pub fn diagonal(data: [f64; SIZE]) -> Self {
        let mut result = Self::ZEROS;
        for i in 0..SIZE {
            result.data[i][i] = data[i]
        }
        result
    }
}

impl Tensor<1, 1> {
    pub fn value(&self) -> f64 {
        self.data[0][0]
    }
}

impl<const R: usize, const C: usize> Add for Tensor<R, C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Self::ZEROS;
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = self.data[i][j] + rhs.data[i][j];
            }
        }
        result
    }
}

impl<const R: usize, const C: usize> AddAssign for Tensor<R, C> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..R {
            for j in 0..C {
                self.data[i][j] += rhs.data[i][j];
            }
        }
    }
}


impl<const R: usize, const C: usize> Display for Tensor<R, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}

impl<const R: usize, const C: usize> Debug for Tensor<R, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}


impl<const R: usize, const C: usize> Default for Tensor<R, C> {
    fn default() -> Self {
        Self::ZEROS
    }
}

impl<const R: usize, const C: usize> Distribution<Tensor<R, C>> for StandardUniform {
    fn sample<RNG: Rng + ?Sized>(&self, rng: &mut RNG) -> Tensor<R, C> {
        let mut result = Tensor::ZEROS;
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = rng.random_range(0.0 ..= 1.0);
            }
        }
        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActivationFunction {
    Identity,
    Sigmoid,
    ReLU,
    McCullochPitt(f64)
}

impl Default for ActivationFunction {
    fn default() -> Self {
        ActivationFunction::Sigmoid
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Layer<const IN: usize, const SIZE: usize> {
    weights: Tensor<SIZE, IN>,
    bias: Tensor<SIZE>,
    activation: [ActivationFunction; SIZE],
}

impl<const IN: usize, const SIZE: usize> Display for Layer<IN, SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Layer<{}, {}>:", IN, SIZE)?;
        writeln!(f, "  Weights:")?;
        self.weights.fmt(f, 4)?;
        writeln!(f, "\n  Bias:")?;
        self.bias.fmt(f, 4)?;
        write!(f, "\n  Activations: {:?}", self.activation)
    }
}


impl<const IN: usize, const SIZE: usize> Layer<IN, SIZE> {
    pub fn new(weights: Tensor<SIZE, IN>, bias: Tensor<SIZE>, activation: [ActivationFunction; SIZE]) -> Self {
        Self { weights, bias, activation }
    }

    pub fn fully_connected(weights: Tensor<SIZE, IN>, bias: Tensor<SIZE>, activation: ActivationFunction) -> Self {
        Self::new(weights, bias, [activation; SIZE])
    }

    pub fn mcculloch_pitt(weights: Tensor<SIZE, IN>, thresholds: [f64; SIZE]) -> Self {
        Self::new(weights, Tensor::ZEROS, thresholds.map(ActivationFunction::McCullochPitt))
    }

    pub fn set_activation(&mut self, activation: ActivationFunction) {
        self.activation = [activation; SIZE]
    }

    const WEIGHTS_SIZE: usize = IN * SIZE;
    pub const TOTAL_SIZE: usize = Self::WEIGHTS_SIZE + SIZE; // weights + biases

    pub fn flatten(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(Self::TOTAL_SIZE);
        result.extend(self.weights.flatten());
        result.extend(self.bias.flatten());
        debug_assert_eq!(result.len(), Self::TOTAL_SIZE, "Layer flattened size mismatch");
        result
    }

    pub fn from_slice(data: &[f64]) -> Self {
        debug_assert_eq!(data.len(), Self::TOTAL_SIZE,
           "Invalid data length for Layer<{}, {}>: expected {}, got {}",
           IN, SIZE, Self::TOTAL_SIZE, data.len()
        );
        Self {
            // First WEIGHTS_SIZE elements are weights
            weights: Tensor::from_slice(&data[..Self::WEIGHTS_SIZE]),
            // Remaining SIZE elements are biases
            bias: Tensor::from_slice(&data[Self::WEIGHTS_SIZE..]),
            // Use default activation function
            activation: [Default::default(); SIZE]
        }
    }

    fn forward(&self, input: &Tensor<IN>) -> Tensor<SIZE> {
        // Perform forward propagation: output = (weights · input) + bias
        let mut result = self.weights.dot(input);
        result += self.bias;
        result.activate_mut(self.activation);
        result
    }

    pub fn backward(&self,
                    input: &Tensor<IN>,
                    output: &Tensor<SIZE>,
                    upstream_gradient: &Tensor<SIZE>
    ) -> (Tensor<SIZE, IN>, Tensor<SIZE>, Tensor<IN>) {
        // First apply activation function derivative
        let mut activation_gradient = *upstream_gradient;
        for i in 0..SIZE {
            activation_gradient.data[i][0] *= match self.activation[i] {
                ActivationFunction::Identity => 1.0,
                ActivationFunction::ReLU => if output.data[i][0] > 0.0 { 1.0 } else { 0.0 },
                ActivationFunction::Sigmoid => {
                    let s = output.data[i][0];
                    s * (1.0 - s) // derivative of sigmoid
                },
                ActivationFunction::McCullochPitt(_) => 0.0, // Not differentiable, treated as 0
            };
        }


        // Calculate gradients
        // dL/dW = dL/dY * X^T
        let mut weight_gradient = Tensor::ZEROS;
        for i in 0..SIZE {
            for j in 0..IN {
                weight_gradient.data[i][j] = activation_gradient.data[i][0] * input.data[j][0];
            }
        }

        // dL/db = dL/dY
        let bias_gradient = activation_gradient;

        // dL/dX = W^T * dL/dY
        let mut input_gradient = Tensor::ZEROS;
        for i in 0..IN {
            for j in 0..SIZE {
                input_gradient.data[i][0] += self.weights.data[j][i] * activation_gradient.data[j][0];
            }
        }

        // TODO type this
        (weight_gradient, bias_gradient, input_gradient)
    }

    pub fn update(&mut self, weight_gradient: &Tensor<SIZE, IN>, bias_gradient: &Tensor<SIZE>, learning_rate: f64) {
        // Update weights: W = W - learning_rate * dL/dW
        for i in 0..SIZE {
            for j in 0..IN {
                self.weights.data[i][j] -= learning_rate * weight_gradient.data[i][j];
            }
        }

        // Update biases: b = b - learning_rate * dL/db
        for i in 0..SIZE {
            self.bias.data[i][0] -= learning_rate * bias_gradient.data[i][0];
        }
    }


}

impl<const IN: usize, const SIZE: usize> Distribution<Layer<IN, SIZE>> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Layer<IN, SIZE> {
        let scale = (2.0 / (IN + SIZE) as f64).sqrt();
        let mut weights = Tensor::ZEROS;
        let mut bias = Tensor::ZEROS;

        // Xavier/Glorot initialization
        for i in 0..SIZE {
            for j in 0..IN {
                weights.data[i][j] = (rng.random::<f64>() * 2.0 - 1.0) * scale;
            }
            bias.data[i][0] = (rng.random::<f64>() * 2.0 - 1.0) * 0.1;
        }
        Layer { weights, bias, activation: [Default::default(); SIZE] }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NeuralNetwork<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> {
    input: Layer<IN, WIDTH>,
    hidden: [Layer<WIDTH, WIDTH>; HIDDEN],
    output: Layer<WIDTH, OUT>,
}

impl<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> Display for NeuralNetwork<IN, HIDDEN, OUT, WIDTH> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "NeuralNetwork<{}, {}, {}, {}>", IN, OUT, WIDTH, HIDDEN)?;
        writeln!(f, "Input {}", self.input)?;

        for (i, layer) in self.hidden.iter().enumerate() {
            writeln!(f, "Hidden[{}] {}", i + 1, layer)?;
        }

        write!(f, "Output {}", self.output)
    }
}

impl<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> NeuralNetwork<IN, HIDDEN, OUT, WIDTH> {
    const INPUT_LAYER_SIZE: usize = Layer::<IN, WIDTH>::TOTAL_SIZE;
    const HIDDEN_LAYER_SIZE: usize = Layer::<WIDTH, WIDTH>::TOTAL_SIZE;
    const OUTPUT_LAYER_SIZE: usize = Layer::<WIDTH, OUT>::TOTAL_SIZE;
    pub const TOTAL_SIZE: usize = Self::INPUT_LAYER_SIZE + HIDDEN * Self::HIDDEN_LAYER_SIZE + Self::OUTPUT_LAYER_SIZE;

    pub fn flatten(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(Self::TOTAL_SIZE);

        // Flatten input layer
        result.extend(self.input.flatten());

        // Flatten hidden layers
        for layer in self.hidden.iter() {
            result.extend(layer.flatten());
        }

        // Flatten output layer
        result.extend(self.output.flatten());

        debug_assert_eq!(result.len(), Self::TOTAL_SIZE, "Network flattened size mismatch");
        result
    }

    pub fn from_slice(data: &[f64]) -> Self {
        debug_assert_eq!(data.len(), Self::TOTAL_SIZE,
             "Invalid data length for NeuralNetwork<{}, {}, {}, {}>: expected {}, got {}",
             IN, HIDDEN, OUT, WIDTH, Self::TOTAL_SIZE, data.len()
        );

        let mut offset = 0;

        // Create input layer
        let input = Layer::from_slice(&data[offset..offset + Self::INPUT_LAYER_SIZE]);
        offset += Self::INPUT_LAYER_SIZE;

        // Create hidden layers
        let mut hidden = Vec::with_capacity(HIDDEN);
        for _ in 0..HIDDEN {
            hidden.push(Layer::from_slice(&data[offset..offset + Self::HIDDEN_LAYER_SIZE]));
            offset += Self::HIDDEN_LAYER_SIZE;
        }
        let hidden = hidden.try_into().unwrap();

        // Create output layer
        let output = Layer::from_slice(&data[offset..offset + Self::OUTPUT_LAYER_SIZE]);

        Self { input, hidden, output }
    }


    pub fn set_input_activation(&mut self, activation: ActivationFunction) {
        self.input.set_activation(activation)
    }

    pub fn set_hidden_activation(&mut self, activation: ActivationFunction) {
        for layer in self.hidden.iter_mut() {
            layer.set_activation(activation);
        }
    }

    pub fn set_output_activation(&mut self, activation: ActivationFunction) {
        self.output.set_activation(activation)
    }

    pub fn set_activation(&mut self, activation: ActivationFunction) {
        self.set_input_activation(activation);
        self.set_hidden_activation(activation);
    }

    pub fn set_default_activation(&mut self) {
        self.set_activation(ActivationFunction::Sigmoid);
        self.set_output_activation(ActivationFunction::Identity);
    }

    pub fn forward(&self, input: &Tensor<IN>) -> Tensor<OUT> {
        let mut current = self.input.forward(input);
        for layer in self.hidden.iter() {
            current = layer.forward(&current);
        }
        self.output.forward(&current)
    }

    pub fn train_step(&mut self, input: &Tensor<IN>, target: &Tensor<OUT>, learning_rate: f64) -> f64 {
        // Store activations during forward pass
        let mut hidden_activations = Vec::with_capacity(HIDDEN);
        let mut hidden_outputs = Vec::with_capacity(HIDDEN);

        // Forward pass

        // input layer
        let initial_activation = *input;
        let mut current = self.input.forward(input);
        let initial_output = current;

        // hidden layers
        for layer in self.hidden.iter() {
            hidden_activations.push(current);
            current = layer.forward(&current);
            hidden_outputs.push(current);
        }

        // output layer
        let final_activation = current;
        let final_output = self.output.forward(&current);

        // Calculate loss and initial gradient
        let mut loss = 0.0;
        let mut output_gradient = Tensor::ZEROS;
        for i in 0..OUT {
            let diff = final_output.data[i][0] - target.data[i][0];
            loss += 0.5 * diff * diff; // MSE loss
            output_gradient.data[i][0] = diff; // derivative of MSE
        }

        // Backward pass
        let (w_grad, b_grad, mut upstream_grad) = self.output.backward(
            &final_activation,
            &final_output,
            &output_gradient
        );
        self.output.update(&w_grad, &b_grad, learning_rate);

        // Backpropagate through hidden layers
        for i in (0..HIDDEN).rev() {
            let (w_grad, b_grad, grad) = self.hidden[i].backward(
                &hidden_activations[i],
                &hidden_outputs[i],
                &upstream_grad
            );
            self.hidden[i].update(&w_grad, &b_grad, learning_rate);
            upstream_grad = grad;
        }

        // Input layer
        let (w_grad, b_grad, _) = self.input.backward(
            &initial_activation,
            &initial_output,
            &upstream_grad
        );
        self.input.update(&w_grad, &b_grad, learning_rate);

        loss
    }

    pub fn train(&mut self,
                 inputs: &[Tensor<IN>],
                 targets: &[Tensor<OUT>],
                 epochs: usize,
                 learning_rate: f64
    ) -> Vec<f64> {
        assert_eq!(inputs.len(), targets.len(), "Number of inputs and targets must match");
        let mut losses = Vec::with_capacity(epochs);

        for _ in 0..epochs {
            let mut epoch_loss = 0.0;

            for (input, target) in inputs.iter().zip(targets.iter()) {
                epoch_loss += self.train_step(input, target, learning_rate);
            }

            epoch_loss /= inputs.len() as f64;
            losses.push(epoch_loss);
        }

        losses
    }

}

impl<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> Distribution<NeuralNetwork<IN, HIDDEN, OUT, WIDTH>> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NeuralNetwork<IN, HIDDEN, OUT, WIDTH> {
        let mut network = NeuralNetwork {
            input: rng.random(),
            hidden: from_fn(|_| rng.random()),
            output: rng.random(),
        };
        network.set_default_activation();
        network
    }
}

pub type TetrisNeuralNetwork = NeuralNetwork<20, 2, 1, 20>;

pub const NEURAL_GENOME_SIZE: usize = TetrisNeuralNetwork::TOTAL_SIZE;
pub type NeuralGenome = Genome<NEURAL_GENOME_SIZE>;

impl Into<NeuralGenome> for TetrisNeuralNetwork {
    fn into(self) -> NeuralGenome {
        let array: [f64; NEURAL_GENOME_SIZE] = self.flatten().try_into().unwrap();
        array.into()
    }
}

impl From<NeuralGenome> for TetrisNeuralNetwork {
    fn from(genome: NeuralGenome) -> Self {
        Self::from_slice(&genome.chromosome().map(Coefficient::into_f64))
    }
}

impl TetrisNeuralNetwork {
    pub fn new(weights: &[f64; NEURAL_GENOME_SIZE]) -> Self {
        let mut network = TetrisNeuralNetwork::from_slice(weights);
        network.set_default_activation();
        network
    }
}

impl Default for TetrisNeuralNetwork {
    fn default() -> Self {
        Self::new(&[-0.141809, 0.819535, -0.310605, 0.149258, -0.412118, 0.097435, -0.437277, -0.018952, 0.697807, 0.232663, 0.559266, -0.623576, 0.916003, 0.778437, 0.704062, -0.499860, 0.129439, 0.815024, 0.332836, 0.031986, 0.778059, -0.135356, -0.600196, 0.158302, -0.776317, -0.284473, -0.533517, -0.025760, -0.853281, 1.000004, 0.269146, 0.683415, -0.637125, 0.364764, 0.574182, 0.678015, 0.276061, -0.534691, 0.014663, -0.590003, 0.533131, -0.150816, -0.518281, -0.593881, -1.145083, -0.301085, -0.251812, -0.595405, 0.351176, -0.313842, -0.312894, -0.000945, -0.292773, 0.827246, -0.855727, 0.129246, 0.252879, 0.565775, 0.743407, -0.542788, -0.481081, -0.642325, -0.386119, -0.809142, 0.710661, -1.104479, -0.112039, -0.321641, -0.507853, -0.724125, -0.212224, -0.977005, 0.143691, -1.090564, -0.059170, 0.280789, -0.616699, -0.911085, -0.605155, -0.745956, 0.822610, -0.962913, -0.618925, 0.837403, -0.624907, 0.652727, -0.464967, -0.550959, 0.015121, 1.012794, 0.779049, 0.799806, 0.415246, 0.333613, 0.741450, 1.017960, 0.650235, 0.681804, 0.740772, 0.156210, -0.177463, -0.914259, 0.843066, 0.793259, 0.674865, -0.119990, -0.742363, 0.479218, -0.323428, -0.691862, -0.211715, 0.931656, -0.902552, 0.466968, 0.689289, 0.109748, 0.412389, 0.915794, 0.409450, -0.314643, -0.902396, 0.528520, -0.374958, 0.069622, 1.000471, -0.348025, -0.726480, 0.171686, -0.548427, -0.887503, 0.727387, 0.055884, 0.038030, -0.234842, -0.769267, 0.464274, -0.263589, -0.684996, -1.109777, -0.317631, -0.671345, -0.530491, 0.364970, 0.397746, 0.254074, 0.846419, 0.740504, -0.253124, 0.703619, -0.417577, 0.507847, 0.755730, 0.783469, 1.244997, 0.220105, -0.561179, -0.221613, 0.063500, 0.531078, -0.389706, 0.689395, -0.392242, 0.041327, 0.651913, -0.049031, 0.378899, 0.299426, 0.711495, -0.924666, -0.644753, 0.752079, 1.098645, -0.324543, 0.064782, -0.099339, 0.216591, 0.741833, 0.522969, -0.120014, 0.701463, -0.596049, 1.026038, 0.062652, 0.741064, 0.338471, 0.024440, -0.129649, -1.061741, -0.557823, -0.396349, 0.469142, -0.410221, 0.093287, 0.487978, 0.721121, -0.642627, 0.105906, 0.378926, 0.689868, 0.211613, -0.957424, -0.629726, -0.517602, -0.044882, -0.295583, -0.483490, -0.105506, -0.963750, 0.619488, 0.492571, -0.737176, -1.051278, 0.117828, -0.335320, -0.847313, -0.933559, -0.550311, -0.706543, 0.152640, -0.335922, -0.829139, -0.376528, 0.309467, 0.335389, -0.959500, 0.223094, 0.053313, 0.691857, -0.575213, -0.139600, 0.130800, 0.837343, -0.242681, 0.488065, 0.906327, 0.644365, 0.672674, 0.732650, 0.411367, -0.021400, -0.295919, 0.676767, 0.748515, -0.655362, -0.284514, -0.055170, -0.825881, -0.263976, 0.100648, -0.725385, -0.247122, -0.930212, -0.412771, 1.126563, 0.095138, 0.362919, 0.641021, 0.487315, 0.325718, 0.179101, -0.669207, 0.524910, 0.613405, 0.011935, -0.739964, 0.015120, -0.078040, 0.571835, -0.902977, 0.104495, -0.824832, 1.155438, 0.146235, 0.824382, 0.997543, -0.256634, -0.383184, 0.728028, 0.520876, 0.048823, 0.928295, 0.803947, -0.061441, -0.287149, -0.769409, -0.790606, -0.370332, 0.523933, -1.339105, 0.090602, 0.251814, 0.786770, -0.364975, 0.323744, 0.089761, 0.626103, 0.082902, -0.159588, -0.398847, -0.837838, 2.136065, 3.112106, 0.220882, 1.135651, 0.756170, -0.004200, 0.006701, 0.218297, -0.177117, 0.309479, 0.190445, -0.327564, -0.645598, -0.337478, -0.061998, 1.427838, -0.023545, 0.187035, 0.301724, 0.222360, -0.625216, 0.357365, 0.100741, 0.901451, -0.355394, 0.297701, 0.030987, -0.205573, 0.357959, 0.806857, 0.019430, 0.917440, 0.612457, 0.304668, 0.005039, 0.224273, 0.241895, 1.037320, 0.783619, -0.375094, -0.090160, -0.895521, -0.552322, -0.306344, 0.446010, 0.511352, 0.554032, -0.715598, 0.540251, -0.487195, -0.369969, 1.177266, 0.526269, 0.486361, 0.926547, -0.045457, -0.191837, 0.215274, 0.326762, -0.242237, -0.180041, 0.230289, -0.779403, 0.465380, -0.096915, 0.327446, -0.732444, 0.012517, 0.717387, -0.781976, -0.258134, 0.437202, 0.359903, 0.502370, 0.902569, -0.077348, 0.807130, -0.356392, 0.104466, 0.474632, 0.823903, -0.847826, 0.153500, -0.036883, 0.263432, -1.080269, -0.169603, -0.425982, -0.368196, 0.622385, 0.928556, -0.522267, -0.077614, 0.705399, 0.504431, 0.743624, 0.662523, 0.797067, 0.743376, -0.080978, -0.374764, 0.228982, -0.043034, -0.335681, -0.362767, 0.942049, -0.049547, -0.419701, 0.791816, 0.618632, 0.186521, 0.116589, -0.793073, -0.176733, 0.815672, 0.803659, 0.209767, -0.778792, 0.939204, 0.572963, -0.945653, 0.449054, 0.653894, 0.209362, 0.345207, 1.011188, 0.879523, 0.000015, -0.097967, 0.323063, -0.114776, 0.669763, 0.442721, -0.598773, 0.708816, -0.419174, 0.000595, -0.348448, -0.704388, -0.033368, 0.787601, -0.140621, -0.039168, 0.249268, 0.418436, -0.651943, 0.457419, -0.658921, -0.141827, 0.537405, 0.582896, -0.031625, 0.439993, 0.547247, 0.218947, 0.560495, -0.893269, -0.606943, -0.643197, -0.618585, 0.506031, 0.806948, -0.261224, -1.064688, 0.934052, -0.135212, -0.860202, -0.724695, -0.862574, 0.927532, -0.897085, -0.314936, -0.743796, -0.573516, -0.545612, -0.855000, -0.922364, -0.802906, -0.830968, 0.356302, -0.354166, -0.448053, -0.899717, 0.067471, 0.399583, 0.065976, -0.108244, 0.579199, 0.526255, -0.832330, -0.809285, -0.813208, 0.367570, 0.803463, 0.029671, 0.425936, -0.096818, -0.158591, -0.169176, 0.364440, 0.805687, 0.152745, -0.412082, 0.842685, -0.761438, -0.539381, -0.957965, 0.052724, -0.612014, -0.829791, -0.028023, -0.207036, -0.728665, -0.888951, 0.382626, 0.671669, -0.501102, 0.339708, -0.179985, -0.386718, 0.561142, 0.053644, 0.957006, 0.483684, 0.843254, -0.850178, 0.012169, -0.848255, -0.947377, 0.824581, 0.707615, 0.803041, -0.015167, -0.057430, 0.167804, 0.758079, 0.606140, 1.067481, 0.073429, -0.614793, 0.601354, 0.531359, 0.476496, 0.486257, 0.699507, 0.116160, -0.614561, 0.501782, -0.632810, 0.075105, -0.046105, -0.477361, 0.442440, -0.607618, -0.995808, -0.430577, -0.948587, 0.005366, -0.614309, 0.559724, -0.733459, 0.264668, -0.676638, 0.289884, -0.143668, 0.158119, -0.211311, 0.469834, -0.252451, 0.712261, 0.610776, 0.352352, -0.685589, 0.112387, -1.073677, -0.404411, 0.301555, -0.044514, -0.531802, -0.355914, -0.342672, 0.328146, 0.644102, 0.985635, -0.320505, -0.489866, -0.226481, -0.754707, -0.830181, -0.212966, -0.288112, -0.430689, 0.866094, -0.111553, -0.738205, -0.103616, 0.900212, 0.796386, -0.131494, 0.702475, 0.332022, -0.626902, -0.137892, 0.851086, 0.118761, -0.750085, 0.921478, 0.245134, -0.463317, -0.783686, -0.189653, -0.966498, -0.051794, -0.845619, 0.954924, -0.905343, 0.975580, -0.997589, -0.291783, 0.810745, -0.024270, -0.606860, -0.571006, -0.802610, 0.979988, 0.451595, -0.162310, -0.040321, -0.774392, -0.364802, -0.241933, -0.945197, -0.781324, -0.606103, 0.003124, 0.379064, 0.686324, 0.096972, -0.763768, 0.678334, 0.200480, 0.403845, 0.655056, -0.251999, -0.432752, -0.482205, -0.598699, -0.555771, -0.148050, 0.872185, 0.847766, 0.597395, 0.189862, -0.463414, -1.144573, 0.534357, -0.367554, -0.989043, 0.589739, -0.179146, 0.028237, -0.420154, 0.613153, -0.194724, 0.115038, -0.371347, -0.437813, -0.477682, 0.771086, 0.870800, -0.421150, 0.752354, -0.403997, -0.640948, -0.268994, -0.499961, -0.532625, 0.386733, 1.187873, -0.083283, 0.523361, 0.136542, 0.554540, 0.036601, -0.440007, -0.968548, 0.386538, 0.325499, -0.989643, 0.455849, 0.126937, 0.642052, 0.377107, -0.130497, -0.313507, -0.895628, 0.702326, -0.333471, -0.273209, 0.002833, 0.675096, 0.077305, -0.105449, -0.182339, 0.674715, -0.199715, 0.574457, 0.145763, -0.871721, -0.269434, -0.789842, -0.379763, 0.514697, -0.835876, -0.791730, -0.409606, 1.045079, 0.457998, -0.838355, 0.578251, -0.510672, -0.477018, 0.064590, 0.149377, 0.528945, -0.628383, -0.840975, -0.020476, 0.606439, -0.292254, 0.466187, -0.697952, 0.375483, -0.948440, -0.609821, 0.638769, 0.085298, -0.504703, -0.531100, 0.302521, -0.463724, -0.769915, -0.849756, -0.453681, 0.714244, 0.392185, 0.057337, 0.587880, -0.413865, 0.721162, 0.066410, 0.285766, 0.897133, 0.130566, 0.203429, -0.029040, -0.490907, -0.569679, 0.456191, 0.117473, -0.656052, 0.606907, 0.480279, -0.096244, -0.676420, 0.868980, -0.005150, -0.257875, 0.001906, 0.192422, -0.122659, -0.092647, -0.252121, -0.340922, 0.244310, -0.963758, -0.393385, 0.368612, -0.634465, -0.616756, -0.192957, 0.020853, -0.635320, 0.428306, -0.631100, 0.843060, 0.395146, 0.879783, -0.645641, -0.249418, 0.860852, -0.853304, -0.634457, 0.394768, -0.369342, 0.795951, 0.523707, -0.087177, -0.342788, -0.430719, -0.004594, -0.602086, -0.873724, 0.413739, 0.602905, -0.470343, 0.219571, 0.790689, 0.914873, 0.269021, 0.105772, 0.174302, -0.157370, -0.271203, 0.061187, 0.995065, 0.524641, 0.352123, -0.836602, -0.235476, -0.547930, -0.395287, -0.694725, 0.631014, 0.915564, 0.945477, -0.457412, 0.164659, -0.405871, -0.883825, 0.530147, -0.893556, -0.589184, 0.603317, -0.130926, -0.668672, -0.696532, -0.297235, -0.543510, -0.516920, -0.410545, -0.683852, 0.839619, -0.809133, -0.493359, 0.051856, 0.878383, 0.696490, -0.364939, 0.908982, -0.046085, 0.413275, 0.223222, 0.994589, -0.186167, 0.960982, -0.598605, 0.952408, 0.057173, -0.328320, 0.982933, -0.310161, -0.588914, 0.388062, -0.386329, 0.583588, 0.213001, 0.733584, -0.899458, 0.371540, -0.123180, -0.670424, 0.241606, -0.374718, -0.105321, -0.042710, 0.049665, -1.140854, 0.156892, 0.852584, -0.465248, -0.918702, -0.295587, 0.328400, -0.061689, -0.852031, -0.360939, 0.296184, 0.569918, -0.762023, -0.365516, 0.279800, 0.235772, -0.315386, 0.169311, -0.828845, -1.280087, -0.361602, -0.812097, 0.504865, 0.386695, -0.823190, 0.157485, -0.504345, 0.808808, 0.316543, 0.765820, 0.690262, 0.562710, -0.788770, 0.389832, -1.003606, 0.833250, -0.444199, -0.386104, -0.524584, 0.074327, -0.184337, -0.441326, 0.178462, -0.407096, 0.081212, -0.985799, 0.439615, -0.158917, -0.869018, -0.895100, 0.930297, 0.399054, -0.781016, -0.399090, -0.888884, -0.024508, 0.296341, 0.423867, 0.855457, -0.347627, -0.361891, 0.029211, 0.969675, 0.387442, -0.938649, -0.588655, -0.865482, -0.892087, -0.708876, 0.726597, 0.713936, 1.025973, -0.613805, -0.481679, 0.501815, 0.513963, -0.391864, 0.478564, 0.944577, 0.173790, 0.225505, 0.145827, -0.583163, -0.172680, -0.077327, 0.480291, 0.204300, 0.414548, 0.511590, -0.191604, 0.653928, 0.904630, 0.763651, -0.574633, -0.862347, 0.854360, 0.104928, 0.929823, -0.436249, -0.627196, 0.670183, 0.145598, -0.017063, -0.080090, -0.016500, -0.057709, 0.297065, -0.259129, 1.027859, -0.067143, 0.400137, 0.178043, 0.681472, -0.377402, -1.101692, -0.216569, -0.183543, -0.071956, -1.057377, -0.253145, -0.855065, 1.021897, 0.222747, 0.050180, 0.395083, -0.070385, 0.347209, 0.032664, 0.768298, -0.816599, 0.023388, -0.022130, -1.118947, -0.869073, 0.828529, -0.363169, 0.599790, 0.237944, 0.803368, -0.422699, 0.342265, -0.865775, 0.443270, -0.087914, 0.844723, -0.718921, -0.373822, -0.909565, -1.052538, 0.526906, -0.519566, 0.765605, 0.944221, -0.984506, 0.561389, 0.566614, -0.332012, -0.684003, 0.851106, 0.507254, -0.743759, -0.336806, -0.680133, -0.090058, -0.360258, 0.257138, -0.058152, -0.471700, -0.624798, -0.290429, 0.017063, -0.393226, -0.655407, 0.404402, -0.634264, 0.927822, 0.437621, 0.926781, -0.431610, -0.692833, -0.117161, -0.681183, -0.702166, 0.285622, -0.337830, -0.342651, -0.426043, -0.049755, -0.787862, -0.560164, -0.726502, 0.578930, 0.464688, 0.382275, -0.325839, -0.641523, -1.091437, -0.969597, -0.523455, 0.842735, -0.210158, 0.117115, 0.929075, -0.332328, 0.624656, -0.615955, 0.776479, -0.769574, -0.566656, -0.278951, 0.289640, -0.554586, 0.651790, 0.474483, -0.262369, 0.773547, -0.566705, -0.329407, 0.824891, -0.434793, 0.310507, 0.220887, -0.450111, -0.326063, -0.601827, -0.742433, 0.630203, -0.940905, -0.315530, 0.347913, 0.650634, -0.529044, -0.508485, 0.525002, 0.488648, 0.211946, 0.241992, -0.092670, 0.678093, 0.045779, -1.097323, 0.410898, 0.444796, 0.339207, -0.997378, -0.139389, 0.364297, 0.215018, 0.626742, 0.165395, -0.767886, -0.333722, 0.462423, 0.564569, 0.685745, -0.908905, -0.764178, -0.507078, 0.091679, -1.010970, -0.671304, -0.309281, -0.673751, -0.484300, -0.553663, 0.071577, -0.114129, 0.639981, -0.555660, 0.711768, -0.045243, 0.238986, 0.129202, -0.167125, -0.234598, -0.885664, -0.758075, -0.527644, 0.780508, -1.075415, -0.051302, -0.182124, -0.180827, 0.754253, 0.008492, 0.511706, -0.973805, -0.134734, 0.464513, -0.655536, -0.436569, -0.411064, -0.522466, 0.297646, -0.228151, -0.131211, 0.395814, 0.048185, -0.504315, 0.244501, -0.245663, 0.473095, -0.312578, -0.735839, 1.124151, 0.430011, 1.232487, 0.825930, 0.093128, -0.034694, -0.533190, 0.966133, 1.081373, 0.187814, 0.117129, 0.835377, 0.349694, -1.000793, -0.328404, -0.412488, -0.171716, -0.794188, 0.471936, 0.006499, 0.658934, 0.149762, 0.779668, -0.387518, -0.629803, 1.364775, -0.121559, -0.473583, 1.163393, 0.016834, -0.714705, 0.539290, 1.090526, -0.879561, -1.001861, 0.182452, 0.168863, 0.210493, 0.103419, -0.770848, -0.715181, 0.572836, 0.170717, 0.616388, -0.614521, -0.456974, 0.284747, -0.883438, 0.676085, 0.259516, -1.187333, 0.423982, 0.964324, 0.181448, -0.793470, 0.540421, 0.290992, 0.717698, 0.800475, 0.810574, 0.809345, -0.313877, 0.160271, -0.057534, 0.537509, 0.120233, 0.691357, -0.606951, 0.069654, 0.515732, 0.051472, 0.947151, 0.269150, -0.189486, 0.025382, -0.779045, -0.090413, 0.815452, 0.608317, 0.766327, 0.444999, -0.150147, 0.472963, 0.166767, 1.184291, -0.078903, -0.639889, 1.014536, 0.231677, 0.964034, -0.645166, 0.299975, -0.969557, -0.357669, -0.801316, -1.028452, 0.327733, -0.774355, 0.152036, 0.517073])
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;
    use super::*;

    #[test]
    fn flatten_tensor() {
        let tensor = Tensor::new([[1., 2., 3.], [4., 5., 6.]]);
        let flat = tensor.flatten();
        let from_flat = Tensor::from_slice(&flat);
        assert_eq!(tensor, from_flat);
    }

    #[test]
    fn flatten_layer() {
        let layer = Layer::fully_connected(
            Tensor::new([[1., 2., 3.], [4., 5., 6.]]),
            Tensor::new([[1.], [2.]]),
            ActivationFunction::default(),
        );
        let flat = layer.flatten();
        let from_flat = Layer::from_slice(&flat);
        assert_eq!(layer, from_flat);
    }

    #[test]
    fn flatten_network() {
        let network = NeuralNetwork {
            input: Layer::fully_connected(
                Tensor::new([[1., 2., 3.], [4., 5., 6.]]),
                Tensor::new([[0.1], [0.2]]),
                ActivationFunction::default(),
            ),
            hidden: [
                Layer::fully_connected(
                    Tensor::new([[7., 8.], [9., 10.]]),
                    Tensor::new([[0.3], [0.4]]),
                    ActivationFunction::default(),
                ),
                Layer::fully_connected(
                    Tensor::new([[11., 12.], [13., 14.]]),
                    Tensor::new([[0.5], [0.6]]),
                    ActivationFunction::default(),
                )
            ],
            output: Layer::fully_connected(
                Tensor::new([[15., 16.]]),
                Tensor::new([[0.7]]),
                ActivationFunction::default(),
            )
        };
        let flattened = network.flatten();
        let expected = vec![
            1., 2., 3., 4., 5., 6., 0.1, 0.2, // input
            7., 8., 9., 10., 0.3, 0.4, // hidden 1
            11., 12., 13., 14., 0.5, 0.6, // hidden 2
            15., 16., 0.7 // output
        ];
        assert_eq!(flattened, expected);

        // Reconstruct network from flattened vector
        let reconstructed = NeuralNetwork::from_slice(&flattened);
        assert_eq!(reconstructed, network);
    }

    #[test]
    fn parse_tetris_network() {
        let weights: [f64; TetrisNeuralNetwork::TOTAL_SIZE] = rand::random();
        let network = TetrisNeuralNetwork::new(&weights);
        let flattened = network.flatten();
        assert_eq!(flattened, weights);
    }

    #[test]
    fn deterministic() {
        let network = TetrisNeuralNetwork::default();
        let mut rng = ChaChaRng::seed_from_u64(42);
        for _ in 0..10 {
            let input = Tensor::vector(rng.random());
            let expected = network.forward(&input).value();
            assert!(!expected.is_nan());
            for _ in 0..10 {
                let result = network.forward(&input).value();
                assert_relative_eq!(result, expected, epsilon = 1e-8);
            }
        }
    }

    #[test]
    fn dot_product() {
        let t1 = Tensor::new([[1., 2., 3.], [4., 5., 6.]]);
        let t2 = Tensor::new([[7., 8.], [9., 10.], [11., 12.]]);
        let result = t1.dot(&t2);
        assert_eq!(result, Tensor::new([[58., 64.], [139., 154.]]));
    }

    #[test]
    fn relu() {
        let mut result = Tensor::new([[-1., 2., 3.], [4., -5., 6.]]);
        result.relu_mut();
        assert_eq!(result, Tensor::new([[0., 2., 3.], [4., 0., 6.]]));
    }

    #[test]
    fn add() {
        let t1 = Tensor::new([[1., 2., 3.], [4., 5., 6.]]);
        let t2 = Tensor::new([[7., 8., 9.], [10., 11., 12.]]);
        let result = t1 + t2;
        assert_eq!(result, Tensor::new([[8., 10., 12.], [14., 16., 18.]]));
    }

    #[test]
    fn fully_connected_layer_forward() {
        let layer = Layer::fully_connected(
            Tensor::new([[1., 2., 3.], [4., 5., 6.]]),
            Tensor::new([[1.], [2.]]),
            ActivationFunction::ReLU,
        );

        let ones = Tensor::ONES;
        let observed = layer.forward(&ones);
        assert_eq!(observed, Tensor::vector([7., 17.]));
    }

    #[test]
    fn test_mcculloch_pitt_network() {
        // network from https://blog.abhranil.net/2015/03/03/training-neural-networks-with-genetic-algorithms/
        let network: NeuralNetwork<2, 0, 1, 2> = NeuralNetwork {
            input: Layer::mcculloch_pitt(
                Tensor::new([[1.0, 1.0], [-1.0, -1.0]]),
                [0.5,-1.5]
            ),
            hidden: [],
            output: Layer::mcculloch_pitt(
                Tensor::new([[1.0, 1.0]]),
                [1.5],
            ),
        };

        for x in [0, 1] {
            for y in [0, 1] {
                let expected = if x == y { 0.0 } else { 1.0 };
                let observed = network.forward(&Tensor::vector([x as f64, y as f64]));
                assert_eq!(observed.value(), expected, "x={}, y={}", x, y);
            }
        }
    }

    #[test]
    fn test_train_x_plus_y() {
        let mut rng = ChaChaRng::seed_from_u64(100);
        let network = train_network::<0, 2>(&mut rng, 100, 1500, |x, y| x + y);
        validate_network(&mut rng, network, 100, |x, y| x + y);
    }

    #[test]
    fn test_train_x_mul_y() {
        let mut rng = ChaChaRng::seed_from_u64(100);
        let network = train_network::<0, 8>(&mut rng, 500, 5000, |x, y| x * y);
        validate_network(&mut rng, network, 100, |x, y| x * y);
    }

    fn random_xy(rng: &mut ChaChaRng) -> (f64, f64) {
        let x = rng.random_range(0. .. 1.);
        let y = rng.random_range(0. .. 1.);
        (x, y)
    }

    fn train_network<const HIDDEN: usize, const WIDTH: usize>(
        rng: &mut ChaChaRng,
        training_set_size: usize,
        epochs: usize,
        function: impl Fn(f64, f64) -> f64
    ) -> NeuralNetwork<2, HIDDEN, 1, WIDTH> {
        // Create a simple network: 2 inputs, 1 output
        let mut network: NeuralNetwork<2, HIDDEN, 1, WIDTH> = rng.random();
        network.set_activation(ActivationFunction::Sigmoid);


        // build training data from random numbers
        let mut inputs = vec![];
        let mut targets = vec![];
        for _ in 0..training_set_size {
            let (x, y) = random_xy(rng);
            inputs.push(Tensor::vector([x, y]));
            targets.push(Tensor::vector([function(x, y)]))
        }

        // Train the network
        network.train(&inputs, &targets, epochs, 0.01);

        network
    }

    fn validate_network<const HIDDEN: usize, const WIDTH: usize>(
        rng: &mut ChaChaRng,
        network: NeuralNetwork<2, HIDDEN, 1, WIDTH>,
        validation_set_size: usize,
        function: impl Fn(f64, f64) -> f64
    ) {
        let mut sum_error = 0.0;
        for _ in 0..validation_set_size {
            let (x, y) = random_xy(rng);
            let expected = function(x, y);
            let observed = network.forward(&Tensor::vector([x, y]));
            sum_error += (expected - observed.value()).abs();
        }

        let mean_error = sum_error / validation_set_size as f64;
        assert_relative_eq!(
            mean_error,
            0.0,
            epsilon = 0.01, // within 1%
        );
    }

}
