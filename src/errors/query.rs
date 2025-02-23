use klickhouse::{ KlickhouseError };

#[derive(Debug)]
pub enum QueryErr {
    KlickhouseError,
    MissingParametersError
}
