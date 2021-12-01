use std::collections::HashMap;

/// (1) You are a TA at a university, and you want to evaluate your student’s homework
/// without executing their (untrusted) code. You decide to write a small
/// web-service that takes bytecode as input, and interprets the results.

/// The bytecode language you need to support includes basic arithmetic and
/// variables. The bytecode language is stack, rather than register based.
/// ByteCode (right) is given for the following pseudo code (left):
///
/// ```
/// fn function_f() -> i32 {
///
///    let x = 1;                   // LOAD_VAL 1
///                                 // WRITE_VAR ‘x’
///
///    let y = 2;                   // LOAD_VAL 2
///                                 // WRITE_VAR ‘y’
///
///    return (x + 1) * y ;         // READ_VAR ‘x’
///                                 // LOAD_VAL 1
///                                 // ADD
///
///                                 // READ_VAR ‘y’
///                                 // MULTIPLY
///
///                                 // RETURN_VALUE
/// }
/// ```
/// Add a data type `ByteCode` that can represent bytecode like in the example
/// above, along with an interpreter for said bytecode. Make sure your bytecode
/// is flat, i.e. not nested.
#[derive(Copy, Clone, PartialEq)]
pub enum ByteCode<'a> {
    /// Unary operation: load a number to the number stack
    LoadVal(i32),
    /// Unary operation: bind the loaded in the stack value to the variable
    WriteVar(&'a str),
    /// Unary operation: load the value binded to the variable to the number stack
    ReadVar(&'a str),
    /// Binary operation: pop the 2 last values of the number stack and add them to the number stack
    Add,
    /// Binary operation: pop the 2 last values of the number stack and subtract them to the number stack
    Subtract,
    /// Binary operation: pop the 2 last values of the number stack and multiply them to the number stack
    Multiply,
    /// Binary operation: pop the 2 last values of the number stack and divide them to the number stack
    Divide,
    /// Binary operation: pop the 2 last values of the number stack and return the remainder of the division to the number stack
    Modulo,
    /// Unary operation: pop the last value of the number stack and repeat `value` times the next ByteCodes until the EndLoop Bytecode
    Loop,
    /// Null operation: marks the end of the current loop
    EndLoop,
}

pub struct Interpreter<'a> {
    /// Values are loaded in the number stack
    number_stack: Vec<i32>,

    /// Variables are binded to their value in a HashMap
    variables_map: HashMap<&'a str, i32>,
}

impl<'a> Default for Interpreter<'a> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a> Interpreter<'a> {
    #[must_use]
    pub fn new() -> Interpreter<'static> {
        Interpreter {
            number_stack: Vec::with_capacity(20),
            variables_map: HashMap::new(),
        }
    }
    pub fn evaluate(&mut self, bytecodes: &[ByteCode<'a>]) -> Result<i32, &'static str> {
        self.evaluate_bytecodes(bytecodes)?;

        match self.number_stack.pop() {
            Some(number) => Ok(number),
            None => Err("Incorrectly formatted expression: no return value."), // We arbitrary expect a return value
        }
    }

    fn evaluate_bytecodes(&mut self, bytecodes: &[ByteCode<'a>]) -> Result<(), &'static str> {
        for (bytecode_index, bytecode) in bytecodes.iter().enumerate() {
            match *bytecode {
                ByteCode::LoadVal(number) => self.number_stack.push(number),
                ByteCode::WriteVar(variable) => self.bind_variable(variable)?,
                ByteCode::ReadVar(variable) => self.read_variable(variable)?,
                ByteCode::Loop => self.repeat(bytecodes, bytecode_index)?,
                ByteCode::EndLoop => (),
                _ => self.binary_calculus(bytecode)?,
            }
        }
        Ok(())
    }

    fn bind_variable(&mut self, variable: &'a str) -> Result<(), &'static str> {
        match self.number_stack.pop() {
            Some(rvalue) => {
                self.variables_map.insert(variable, rvalue);
                Ok(())
            }
            None => Err("Trying to bind variable without value."),
        }
    }

    fn read_variable(&mut self, variable: &'a str) -> Result<(), &'static str> {
        match self.variables_map.get(variable) {
            Some(&number) => {
                self.number_stack.push(number);
                Ok(())
            }
            None => Err("Trying to read variable that does not exist."),
        }
    }

    /// Repeat a set of instructions `x` times where `x` is the top value of the number stack
    fn repeat(
        &mut self,
        inputs: &[ByteCode<'a>],
        bytecode_index: usize,
    ) -> Result<(), &'static str> {
        let endloop_bytecode_index = Self::next_endloop_bytecode(&inputs[bytecode_index..])?;
        let time_number_to_repeat = match self.number_stack.pop() {
            Some(number) => number as usize,
            None => return Err("A number is required to use Loop."),
        };

        for _ in 0..time_number_to_repeat {
            self.evaluate_bytecodes(
                &inputs[bytecode_index + 1..bytecode_index + endloop_bytecode_index],
            )?;
        }

        Ok(())
    }

    /// Given a collection of bytecodes, find the next EndLoop bytecode
    fn next_endloop_bytecode(bytecodes: &[ByteCode]) -> Result<usize, &'static str> {
        let endloop_bytecodes = bytecodes
            .iter()
            .enumerate()
            .filter(|(_, &y)| y == ByteCode::EndLoop)
            .map(|(x, _)| x)
            .collect::<Vec<usize>>();

        match endloop_bytecodes.first() {
            Some(&first_endloop_bytecode_index) => Ok(first_endloop_bytecode_index),
            None => Err("There is no EndLoop instruction associated to the previous Loop."),
        }
    }

    fn binary_calculus(&mut self, bytecode: &ByteCode) -> Result<(), &'static str> {
        let first_operand = self.number_stack.pop();
        let second_operand = self.number_stack.pop();

        self.number_stack.push(Self::perform_binary_operation(
            bytecode,
            first_operand,
            second_operand,
        )?);

        Ok(())
    }

    const fn perform_binary_operation(
        bytecode: &ByteCode,
        first_operand: Option<i32>,
        second_operand: Option<i32>,
    ) -> Result<i32, &'static str> {
        match (bytecode, first_operand, second_operand) {
            (ByteCode::Add, Some(a), Some(b)) => Ok(b + a),
            (ByteCode::Subtract, Some(a), Some(b)) => Ok(b - a),
            (ByteCode::Multiply, Some(a), Some(b)) => Ok(b * a),
            (ByteCode::Divide, Some(a), Some(b)) => Ok(b / a),
            (ByteCode::Modulo, Some(a), Some(b)) => Ok(b % a),
            _ => Err("Incorrectly formatted expression: expecting 2 operands."),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ByteCode, Interpreter};

    #[test]
    fn problem_example_test() {
        let mut interpreter = Interpreter::new();
        //    x = 1
        //    y = 2
        //    return (x + 1) * y
        let bytecodes = [
            ByteCode::LoadVal(1),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(2),
            ByteCode::WriteVar("y"),
            ByteCode::ReadVar("x"),
            ByteCode::LoadVal(1),
            ByteCode::Add,
            ByteCode::ReadVar("y"),
            ByteCode::Multiply,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(4));
    }

    #[test]
    fn simple_addition() {
        //    x = 1
        //    y = 2
        //    return x + y
        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(2),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(3),
            ByteCode::WriteVar("y"),
            ByteCode::ReadVar("x"),
            ByteCode::ReadVar("y"),
            ByteCode::Add,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(5));
    }

    #[test]
    fn simple_substraction() {
        //    x = 1
        //    y = 2
        //    return x - y

        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(2),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(3),
            ByteCode::WriteVar("y"),
            ByteCode::ReadVar("x"),
            ByteCode::ReadVar("y"),
            ByteCode::Subtract,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(-1));
    }

    #[test]
    fn simple_division() {
        //    x = 10
        //    y = 5
        //    return x / y

        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(10),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::WriteVar("y"),
            ByteCode::ReadVar("x"),
            ByteCode::ReadVar("y"),
            ByteCode::Divide,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(2));
    }

    #[test]
    fn simple_multiplication() {
        //    x = 10
        //    y = 5
        //    return x * y
        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(10),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::WriteVar("y"),
            ByteCode::ReadVar("x"),
            ByteCode::ReadVar("y"),
            ByteCode::Multiply,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(50));
    }

    #[test]
    fn simple_modulo() {
        //    x = 10
        //    y = 5
        //    return x % y
        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(10),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::WriteVar("y"),
            ByteCode::ReadVar("x"),
            ByteCode::ReadVar("y"),
            ByteCode::Modulo,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(0));
    }

    #[test]
    fn double_binding() {
        //    x = 10
        //    x = 5
        //    return x

        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(10),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::WriteVar("x"),
            ByteCode::ReadVar("x"),
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(5));
    }

    #[test]
    fn self_asignement() {
        //    x = 10
        //    x = x + 5
        //    return x
        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(10),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::ReadVar("x"),
            ByteCode::Add,
            ByteCode::WriteVar("x"),
            ByteCode::ReadVar("x"),
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(15));
    }

    #[test]
    fn long_expression() {
        //    x = 10
        //    x = x + 5
        //    z = 4
        //    cacao = 15
        //    return ((x * 5) / cacao) - z

        let mut interpreter = Interpreter::new();

        let bytecodes = [
            ByteCode::LoadVal(10),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::ReadVar("x"),
            ByteCode::Add,
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(4),
            ByteCode::WriteVar("z"),
            ByteCode::LoadVal(15),
            ByteCode::WriteVar("cacao"),
            ByteCode::ReadVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::Multiply,
            ByteCode::ReadVar("cacao"),
            ByteCode::Divide,
            ByteCode::ReadVar("z"),
            ByteCode::Subtract,
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(1));
    }

    #[test]
    fn simple_while_loop() {
        let mut interpreter = Interpreter::new();
        //    x = 0
        //    loop 5
        //      x = x + 5
        //    endloop
        //    return x
        let bytecodes = [
            ByteCode::LoadVal(0),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::Loop,
            ByteCode::ReadVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::Add,
            ByteCode::WriteVar("x"),
            ByteCode::EndLoop,
            ByteCode::ReadVar("x"),
        ];
        assert_eq!(interpreter.evaluate(&bytecodes), Ok(30));
    }

    #[test]
    fn no_value_returned_error() {
        let mut interpreter = Interpreter::new();
        let bytecodes = [];

        assert_eq!(
            interpreter.evaluate(&bytecodes),
            Err("Incorrectly formatted expression: no return value.")
        );
    }

    #[test]
    fn try_to_bind_without_value_error() {
        let mut interpreter = Interpreter::new();

        let bytecodes = [ByteCode::WriteVar("x")];

        assert_eq!(
            interpreter.evaluate(&bytecodes),
            Err("Trying to bind variable without value.")
        );
    }
    #[test]
    fn try_to_read_value_not_existing_error() {
        let mut interpreter = Interpreter::new();

        let bytecodes = [ByteCode::ReadVar("x")];

        assert_eq!(
            interpreter.evaluate(&bytecodes),
            Err("Trying to read variable that does not exist.")
        );
    }
    #[test]
    fn no_value_loaded_before_loop_error() {
        let mut interpreter = Interpreter::new();
        //    x = 0
        //    loop ?
        //      x = x + 5
        //    endloop
        //    return x
        let bytecodes = [
            ByteCode::LoadVal(0),
            ByteCode::WriteVar("x"),
            ByteCode::Loop,
            ByteCode::ReadVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::Add,
            ByteCode::WriteVar("x"),
            ByteCode::EndLoop,
            ByteCode::ReadVar("x"),
        ];

        assert_eq!(
            interpreter.evaluate(&bytecodes),
            Err("A number is required to use Loop.")
        );
    }
    #[test]
    fn no_endloop_bytecode_error() {
        let mut interpreter = Interpreter::new();
        //    x = 0
        //    loop 5
        //      x = x + 5
        //
        //    return x
        let bytecodes = [
            ByteCode::LoadVal(0),
            ByteCode::WriteVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::Loop,
            ByteCode::ReadVar("x"),
            ByteCode::LoadVal(5),
            ByteCode::Add,
            ByteCode::WriteVar("x"),
            ByteCode::ReadVar("x"),
        ];

        assert_eq!(
            interpreter.evaluate(&bytecodes),
            Err("There is no EndLoop instruction associated to the previous Loop.")
        );
    }
    #[test]
    fn not_enough_operands_in_binary_operation_error() {
        let mut interpreter = Interpreter::new();
        //    return 5 +
        let bytecodes = [ByteCode::LoadVal(5), ByteCode::Add];

        assert_eq!(
            interpreter.evaluate(&bytecodes),
            Err("Incorrectly formatted expression: expecting 2 operands.")
        );
    }
}
