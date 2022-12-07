#![allow(dead_code)]
use clap::Parser;

extern crate google_sheets4 as sheets4;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;

use yup_oauth2::ServiceAccountAuthenticator;
use yup_oauth2::authenticator::Authenticator;

use sheets4::Sheets;
use google_sheets4::api::ValueRange;

use hyper_rustls::HttpsConnector;
use hyper::client::HttpConnector;

#[derive(Parser)]
struct Cli {
    #[clap(long)]
    workbook_id: String,
    #[clap(long)]
    range: String,
    #[clap(long)]
    credentials_path: String // std::path::PathBuf
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    dump_google_sheet(&args.credentials_path,
          &args.workbook_id,
          &args.range)
        .await
}

async fn dump_google_sheet(creds_path: &str, workbook_id: &str, range: &str) {
    let hub = make_hub(creds_path).await;

    let result = hub.spreadsheets()
        .values_get(workbook_id, range)
        .doit().await;

    match result {
        Err(e) => {
            println!("{}", e);
        },
        Ok((_response, ValueRange { values: Some(value_range), .. })) => {
            for row in value_range {
                println!("{:?}", row);
            }
        },
        Ok(_) => {
            println!("error or malformed spreadsheet");
        }
    }
}

async fn get_service_account(creds_path : &str) ->
    Authenticator<HttpsConnector<HttpConnector>>
{
    let creds = yup_oauth2::read_service_account_key(creds_path)
        .await
        .unwrap();
    let auth = ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .unwrap();
    auth
}

async fn make_hub(creds_path : &str) -> Sheets<HttpsConnector<HttpConnector>> {
    let auth = get_service_account(creds_path).await;
    let hub = Sheets::new(
        hyper::Client::builder()
            .build(hyper_rustls::HttpsConnectorBuilder::new()
                   .with_native_roots()
                   .https_or_http()
                   .enable_http1()
                   .enable_http2()
                   .build()
            ), auth
    );
    hub
}
