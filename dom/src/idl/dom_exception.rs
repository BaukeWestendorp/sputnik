// SPECLINK: https://webidl.spec.whatwg.org/#idl-DOMException
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct DomException {
    pub name: ErrorName,
    pub message: &'static str,
    pub code: u16,
}

impl DomException {
    fn new(name: ErrorName, message: &'static str, code: u16) -> Self {
        Self {
            name,
            message,
            code,
        }
    }
}

impl From<ErrorName> for DomException {
    fn from(value: ErrorName) -> Self {
        match value {
            ErrorName::HierarchyRequestError => DomException::new(
                value,
                "The operation would yield an incorrect node tree.",
                DomException::HIERARCHY_REQUEST_ERR,
            ),
            ErrorName::WrongDocumentError => DomException::new(
                value,
                "The object is in the wrong document.",
                DomException::WRONG_DOCUMENT_ERR,
            ),
            ErrorName::InvalidCharacterError => DomException::new(
                value,
                "The string contains invalid characters.",
                DomException::INVALID_CHARACTER_ERR,
            ),
            ErrorName::NoModificationAllowedError => DomException::new(
                value,
                "The object can not be modified.",
                DomException::NO_MODIFICATION_ALLOWED_ERR,
            ),
            ErrorName::NotFoundError => DomException::new(
                value,
                "The object can not be found here.",
                DomException::NOT_FOUND_ERR,
            ),
            ErrorName::NotSupportedError => DomException::new(
                value,
                "The operation is not supported.",
                DomException::NOT_SUPPORTED_ERR,
            ),
            ErrorName::InUseAttributeError => DomException::new(
                value,
                "The attribute is in use by another element.",
                DomException::INUSE_ATTRIBUTE_ERR,
            ),
            ErrorName::InvalidStateError => DomException::new(
                value,
                "The object is in an invalid state.",
                DomException::INVALID_STATE_ERR,
            ),
            ErrorName::SyntaxError => DomException::new(
                value,
                "The string did not match the expected pattern.",
                DomException::SYNTAX_ERR,
            ),
            ErrorName::InvalidModificationError => DomException::new(
                value,
                "The object can not be modified in this way.",
                DomException::INVALID_MODIFICATION_ERR,
            ),
            ErrorName::NamespaceError => DomException::new(
                value,
                "The operation is not allowed by Namespaces in XML.",
                DomException::NAMESPACE_ERR,
            ),
            ErrorName::SecurityError => DomException::new(
                value,
                "The operation is insecure.",
                DomException::SECURITY_ERR,
            ),
            ErrorName::NetworkError => DomException::new(
                value,
                "A network error occurred.",
                DomException::NETWORK_ERR,
            ),
            ErrorName::AbortError => {
                DomException::new(value, "The operation was aborted.", DomException::ABORT_ERR)
            }
            ErrorName::QuotaExceededError => {
                DomException::new(value, "", DomException::QUOTA_EXCEEDED_ERR)
            }
            ErrorName::TimeoutError => DomException::new(value, "", DomException::TIMEOUT_ERR),
            ErrorName::InvalidNodeTypeError => {
                DomException::new(value, "", DomException::INVALID_NODE_TYPE_ERR)
            }
            ErrorName::DataCloneError => DomException::new(value, "", DomException::DATA_CLONE_ERR),
            ErrorName::EncodingError => DomException::new(value, "", 0),
            ErrorName::NotReadableError => DomException::new(value, "", 0),
            ErrorName::UnknownError => DomException::new(value, "", 0),
            ErrorName::ConstraintError => DomException::new(value, "", 0),
            ErrorName::DataError => DomException::new(value, "", 0),
            ErrorName::TransactionInactiveError => DomException::new(value, "", 0),
            ErrorName::ReadOnlyError => DomException::new(value, "", 0),
            ErrorName::VersionError => DomException::new(value, "", 0),
            ErrorName::OperationError => DomException::new(value, "", 0),
            ErrorName::NotAllowedError => DomException::new(value, "", 0),
            ErrorName::OptOutError => DomException::new(value, "", 0),
        }
    }
}

impl DomException {
    pub const INDEX_SIZE_ERR: u16 = 1;
    pub const DOMSTRING_SIZE_ERR: u16 = 2;
    pub const HIERARCHY_REQUEST_ERR: u16 = 3;
    pub const WRONG_DOCUMENT_ERR: u16 = 4;
    pub const INVALID_CHARACTER_ERR: u16 = 5;
    pub const NO_DATA_ALLOWED_ERR: u16 = 6;
    pub const NO_MODIFICATION_ALLOWED_ERR: u16 = 7;
    pub const NOT_FOUND_ERR: u16 = 8;
    pub const NOT_SUPPORTED_ERR: u16 = 9;
    pub const INUSE_ATTRIBUTE_ERR: u16 = 10;
    pub const INVALID_STATE_ERR: u16 = 11;
    pub const SYNTAX_ERR: u16 = 12;
    pub const INVALID_MODIFICATION_ERR: u16 = 13;
    pub const NAMESPACE_ERR: u16 = 14;
    pub const INVALID_ACCESS_ERR: u16 = 15;
    pub const VALIDATION_ERR: u16 = 16;
    pub const TYPE_MISMATCH_ERR: u16 = 17;
    pub const SECURITY_ERR: u16 = 18;
    pub const NETWORK_ERR: u16 = 19;
    pub const ABORT_ERR: u16 = 20;
    pub const URL_MISMATCH_ERR: u16 = 21;
    pub const QUOTA_EXCEEDED_ERR: u16 = 22;
    pub const TIMEOUT_ERR: u16 = 23;
    pub const INVALID_NODE_TYPE_ERR: u16 = 24;
    pub const DATA_CLONE_ERR: u16 = 25;
}

// SPECLINK: https://webidl.spec.whatwg.org/#dfn-error-names-table
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum ErrorName {
    HierarchyRequestError,
    WrongDocumentError,
    InvalidCharacterError,
    NoModificationAllowedError,
    NotFoundError,
    NotSupportedError,
    InUseAttributeError,
    InvalidStateError,
    SyntaxError,
    InvalidModificationError,
    NamespaceError,
    SecurityError,
    NetworkError,
    AbortError,
    QuotaExceededError,
    TimeoutError,
    InvalidNodeTypeError,
    DataCloneError,
    EncodingError,
    NotReadableError,
    UnknownError,
    ConstraintError,
    DataError,
    TransactionInactiveError,
    ReadOnlyError,
    VersionError,
    OperationError,
    NotAllowedError,
    OptOutError,
}
