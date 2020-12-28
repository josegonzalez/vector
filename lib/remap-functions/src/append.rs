use remap::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Append;

impl Function for Append {
    fn identifier(&self) -> &'static str {
        "append"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                accepts: |v| matches!(v, Value::Array(_)),
                required: true,
            },
            Parameter {
                keyword: "item",
                accepts: |_| true,
                required: true,
            },
        ]
    }

    fn compile(&self, mut arguments: ArgumentList) -> Result<Box<dyn Expression>> {
        let value = arguments.required("value")?.boxed();
        let item = arguments.required("item")?.boxed();

        Ok(Box::new(AppendFn { value, item }))
    }
}

#[derive(Debug, Clone)]
struct AppendFn {
    value: Box<dyn Expression>,
    item: Box<dyn Expression>,
}

impl Expression for AppendFn {
    fn execute(&self, state: &mut state::Program, object: &mut dyn Object) -> Result<Value> {
        let mut list = self.value.execute(state, object)?.try_array()?;
        let item = self.item.execute(state, object)?;

        list.push(item);

        Ok(list.into())
    }

    fn type_def(&self, state: &state::Compiler) -> TypeDef {
        use value::Kind;

        self.value
            .type_def(state)
            .fallible_unless(Kind::Array)
            .merge(self.item.type_def(state))
            .with_constraint(Kind::Array)
            .with_inner_type(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use value::Kind;

    test_type_def![
        value_array_infallible {
            expr: |_| AppendFn {
                value: Array::from(vec!["foo", "bar"]).boxed(),
                item: Literal::from("baz").boxed(),
            },
            def: TypeDef { kind: Kind::Array, ..Default::default() },
        }

        value_non_array_fallible {
            expr: |_| AppendFn {
                value: Literal::from(27).boxed(),
                item: Literal::from("foo").boxed(),
            },
            def: TypeDef { kind: Kind::Array, fallible: true, ..Default::default() },
        }
    ];

    test_function![
        append => Append;

        empty_array {
            args: func_args![value: value!([]), item: value!("foo")],
            want: Ok(value!(["foo"])),
        }

        new_item {
            args: func_args![value: value!([11, false, 42.5]), item: value!("foo")],
            want: Ok(value!([11, false, 42.5, "foo"])),
        }

        already_exists_item {
            args: func_args![value: value!([11, false, 42.5]), item: value!(42.5)],
            want: Ok(value!([11, false, 42.5, 42.5])),
        }
    ];
}