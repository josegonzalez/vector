use lazy_static::lazy_static;
use regex::Regex;
use remap::prelude::*;

lazy_static! {
    // This pattern considers each of these as nullish:
    // * Whitespace-only strings of any length
    // * Newline (\n)
    // * Carriage return (\r)
    // * Single dash (-)
    static ref NULLISH_STRING_PATTERN: Regex = Regex::new(r"^(\s*|\\n|\\r|\-)$").unwrap();
}

#[derive(Clone, Copy, Debug)]
pub struct IsNullish;

impl Function for IsNullish {
    fn identifier(&self) -> &'static str {
        "is_nullish"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            accepts: |v| matches!(v, Value::Bytes(_) | Value::Null),
            required: true,
        }]
    }

    fn compile(&self, mut arguments: ArgumentList) -> Result<Box<dyn Expression>> {
        let value = arguments.required("value")?.boxed();

        Ok(Box::new(IsNullishFn { value }))
    }
}

#[derive(Clone, Debug)]
struct IsNullishFn {
    value: Box<dyn Expression>,
}

impl Expression for IsNullishFn {
    fn execute(&self, state: &mut state::Program, object: &mut dyn Object) -> Result<Value> {
        match self.value.execute(state, object)? {
            Value::Bytes(v) => {
                let s = &String::from_utf8_lossy(&v)[..];

                let matches = NULLISH_STRING_PATTERN.is_match(s);

                Ok(matches.into())
            }
            Value::Null => Ok(true.into()),
            _ => Err("input must be a string or null".into()),
        }
    }

    fn type_def(&self, state: &state::Compiler) -> TypeDef {
        use value::Kind;

        self.value
            .type_def(state)
            .fallible_unless(Kind::Bytes | Kind::Null)
            .with_constraint(Kind::Boolean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use value::Kind;

    test_type_def![
        string_infallible {
            expr: |_| IsNullishFn {
                value: Literal::from("some string").boxed(),
            },
            def: TypeDef {
                fallible: false,
                kind: value::Kind::Boolean,
                ..Default::default()
            },
        }

        null_infallible {
            expr: |_| IsNullishFn {
                value: Literal::from(()).boxed(),
            },
            def: TypeDef {
                fallible: false,
                kind: value::Kind::Boolean,
                ..Default::default()
            },
        }

        integer_fallible {
            expr: |_| IsNullishFn {
                value: Literal::from(42).boxed(),
            },
            def: TypeDef {
                fallible: true,
                kind: value::Kind::Boolean,
                ..Default::default()
            },
        }

        array_fallible {
            expr: |_| IsNullishFn {
                value: Array::from(vec!["foo"]).boxed(),
            },
            def: TypeDef {
                fallible: true,
                kind: value::Kind::Boolean,
                inner_type_def: Some(TypeDef { kind: Kind::Bytes, ..Default::default() }.boxed())
            },
        }
    ];

    test_function![
        is_nullish => IsNullish;

        empty_string {
            args: func_args![value: value!("")],
            want: Ok(value!(true)),
        }

        single_space_string {
            args: func_args![value: value!(" ")],
            want: Ok(value!(true)),
        }

        multi_space_string {
            args: func_args![value: value!("     ")],
            want: Ok(value!(true)),
        }

        newline_string {
            args: func_args![value: value!("\n")],
            want: Ok(value!(true)),
        }

        carriage_return_string {
            args: func_args![value: value!("\r")],
            want: Ok(value!(true)),
        }

        dash_string {
            args: func_args![value: value!("-")],
            want: Ok(value!(true)),
        }

        null {
            args: func_args![value: value!(null)],
            want: Ok(value!(true)),
        }

        non_empty_string {
            args: func_args![value: value!("hello world")],
            want: Ok(value!(false)),
        }
    ];
}
