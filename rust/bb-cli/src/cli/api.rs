use bb_core::{ApiRequest, Request};
use clap::Args;

#[derive(Debug, Args)]
pub(super) struct ApiArgs {
    #[arg(long, default_value = "GET")]
    /// HTTP method
    pub(super) method: String,
    /// Request body file ("-" for stdin)
    #[arg(long)]
    pub(super) input: Option<String>,
    #[arg(long)]
    /// Follow every pagination `next` link
    pub(super) paginate: bool,
    #[arg(long)]
    /// Authentication profile name
    pub(super) profile: Option<String>,
    #[arg(long)]
    /// Bitbucket Cloud API filter expression
    pub(super) q: Option<String>,
    #[arg(long)]
    /// Bitbucket Cloud API sort expression
    pub(super) sort: Option<String>,
    #[arg(long)]
    /// Bitbucket Cloud API partial-response fields
    pub(super) fields: Option<String>,
    /// Relative API path or absolute Bitbucket API URL
    pub(super) endpoint: Option<String>,
}

pub(super) fn map_request(args: ApiArgs) -> Request {
    Request::Api(ApiRequest {
        method: args.method,
        input: args.input,
        paginate: args.paginate,
        profile: args.profile,
        q: args.q,
        sort: args.sort,
        fields: args.fields,
        endpoint: args.endpoint,
    })
}
