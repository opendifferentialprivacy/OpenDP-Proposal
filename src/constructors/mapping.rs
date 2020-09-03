use crate::Transformation;
use crate::base::{Domain, Data, Value};
use crate::metric::DataDistance;

fn make_unary_t_map(
    input_domain: Domain, transform: Transformation,
    input_column: String, output_column: String
) -> Result<Transformation, crate::Error> {

    let output_domain = match input_domain.clone() {
        Domain::Dataframe(mut dataframe_domain) => {
            match dataframe_domain.columns.get(&input_column).cloned() {
                Some(column_domain) => {
                    if column_domain != transform.input_domain {
                        return Err(crate::Error::DomainMismatch)
                    }
                    dataframe_domain.columns.insert(output_column.clone(), transform.output_domain);
                    Domain::Dataframe(dataframe_domain)
                },
                None => return Err(crate::Error::InvalidDomain)
            }
        }
        _ => return Err(crate::Error::Raw("mapper only works on dataframes"))
    };

    let Transformation {
        stability_relation,
        function,
        hint,
        ..
    } = transform;

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| stability_relation(in_dist, out_dist)),
        function: Box::new(move |data: Data| match data {
            Data::Literal(value) => match value {
                Value::Dataframe(mut dataframe) => match dataframe.columns.get(&input_column).cloned() {
                    Some(column) => {
                        let new_column = match (function)(Data::Literal(column))? {
                            Data::Literal(column) => column,
                            _ => return Err(crate::Error::InvalidDomain)
                        };

                        dataframe.columns.insert(output_column.clone(), new_column);
                        Ok(Data::Literal(Value::Dataframe(dataframe)))
                    },
                    None => Err(crate::Error::InvalidDomain)
                },
                _ => Err(crate::Error::InvalidDomain)
            },
            _ => return Err(crate::Error::NotImplemented)
        }),
        hint
    })
}