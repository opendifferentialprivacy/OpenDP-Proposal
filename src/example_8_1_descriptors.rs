use std::collections::HashMap;


// questions on the overleaf P.44
// how do you check that it is a valid value without violating privacy?
// why have a default value?


enum Domain {
    MultiSet(MultiSet),
    Dataframe(DataframeDomain),
    Partition(PartitionDomain),
}


struct AtomicType {
    raw_type: RawType,
    non_null: bool,
    nature: Option<Nature>
}

struct MultiSet {
    atomic_type: Vec<AtomicType>,
    number_rows: i64,
    is_not_empty: bool,
    group_id: Vec<GroupId>,
    distance: Distance,
    is_vector: bool
}

struct DataframeDomain { values: HashMap<IndexKey, Domain> }
struct PartitionDomain { values: HashMap<IndexKey, Domain> }

struct DirectedGraphDomain {
    maximum_degree: Option<i64>,
    distance: Distance,
}

enum Nature {
    Continuous(ContinuousNature),
    Categorical(CategoricalNature),
}

enum ContinuousNature {
    Float(f64, f64),
    Int(i64, i64),
}

enum CategoricalNature {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Bool(Vec<bool>),
}

enum RawType {
    Str,
    Float,
    Int,
    Bool,
    Unknown,
}

struct GroupId {
    partition_id: i64,
    index: IndexKey,
}

enum IndexKey {
    Bool(bool),
    Int(i64),
    Str(String),
    Tuple(Vec<IndexKey>),
}


enum Distance {
    Hamming(u32),
    Symmetric(u32),
    L1(f64),
    L2(f64),
}

enum PrivacyLoss {
    Pure(f64),
    Approximate(f64, f64)
}

struct InteractiveMeasurement {
    input_domain: Domain,
    privacy_loss: PrivacyLoss
}

// fn adaptive_composition(input_domain: Domain, privacy_loss: PrivacyLoss) {
//     InteractiveMeasurement {
//         input_domain,
//         privacy_loss
//     }
// }


fn make_clamp_float(domain: Domain, lower: f64, upper: f64) -> Result<Domain, &'static str> {
    let mut domain = match domain {
        Domain::Vector(domain) => domain,
        _ => return Err("clamp must be performed on a vector")
    };

    if domain.data_type != RawType::Float {
        return Err("clamp_float needs a float atomic type");
    }

    domain.nature = Some(Nature::Continuous(ContinuousNature::Float(lower, upper)));

    return Ok(Domain::Vector(domain));
}
