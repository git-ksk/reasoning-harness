pub mod external_command;
pub mod gemma;
pub mod mistral;
pub mod nvidia;

pub use gemma::GoogleAdapter;
pub use mistral::MistralAdapter;
pub use nvidia::NvidiaAdapter;

pub use external_command::{
    EXTERNAL_COMMAND_RESOLVER_ID, EXTERNAL_RESOLVER_REQUEST_SCHEMA,
    EXTERNAL_RESOLVER_RESPONSE_SCHEMA, ExternalCommandResolver, ExternalCommandResolverConfig,
};
