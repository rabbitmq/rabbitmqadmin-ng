use rabbitmq_http_client::blocking::Error;
use rabbitmq_http_client::responses::DefinitionSet;

pub type Result<T> = std::result::Result<T, Error>;



pub trait Tranformation {
    fn transform(&self: Self, &defs: &mut DefinitionSet) -> Result<&DefinitionSet>;
}

pub struct RemoveClassicQueueMirroringTransformation<'a> {
    pub definition_set: &'a DefinitionSet
}

impl Tranformation for RemoveClassicQueueMirroringTransformation {
    fn transform(defs: &mut DefinitionSet) -> Result<&DefinitionSet> {
        todo!()
    }
}