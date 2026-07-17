use forsl::uri::AppUri;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveRoute {
    pub uri: AppUri,
}
