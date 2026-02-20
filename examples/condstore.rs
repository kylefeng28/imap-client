use imap_client::client::tokio::Client;
use imap_next::imap_types::{
    command::FetchModifier,
    core::Vec1,
    fetch::{Macro, MacroOrMessageDataItemNames},
    search::SearchKey,
    sequence::SequenceSet,
};
use std::num::NonZero;

const USAGE: &str =
    "USAGE: cargo run --features condstore --example=condstore -- <host> <port> <username> <password> [cached_modseq]";

#[tokio::main]
async fn main() {
    let (host, port, username, password, cached_modseq) = {
        let mut args = std::env::args();
        let _ = args.next();

        let host = args.next().expect(USAGE);
        let port = str::parse::<u16>(&args.next().expect(USAGE)).unwrap();
        let username = args.next().expect(USAGE);
        let password = args.next().expect(USAGE);
        let cached_modseq = args.next().and_then(|s| s.parse::<u64>().ok());

        (host, port, username, password, cached_modseq)
    };

    let mut client = Client::rustls(host, port, false, None).await.unwrap();

    client.authenticate_plain(username, password).await.unwrap();

    #[cfg(feature = "condstore")]
    {
        if client.state.ext_condstore_supported() {
            let enabled = client.enable_condstore_if_supported().await.unwrap();
            println!("CONDSTORE enabled via ENABLE command: {}", enabled);
        } else {
            println!("CONDSTORE is not supported by the server");
        }

        // Select mailbox with CONDSTORE enabled
        let select_data = client.select("inbox").await.unwrap();
        println!("\nSelect data: {select_data:?}");

        if let Some(modseq) = select_data.highest_modseq {
            println!("\nCONDSTORE is active (HIGHESTMODSEQ: {})", modseq);

            // Determine test MODSEQ: use cached value if provided, otherwise subtract 200 from HIGHESTMODSEQ
            let test_modseq = if let Some(cached) = cached_modseq {
                println!("Using cached MODSEQ: {}", cached);
                std::num::NonZeroU64::new(cached).unwrap()
            } else {
                let offset = 200;
                let calculated = modseq.get().saturating_sub(offset);
                println!(
                    "No cached MODSEQ provided, using HIGHESTMODSEQ - {} = {}",
                    offset, calculated
                );
                std::num::NonZeroU64::new(calculated).unwrap()
            };

            println!("\n--- Testing FETCH CHANGEDSINCE {} ---", test_modseq);
            let modifiers = vec![FetchModifier::ChangedSince(
                NonZero::new(test_modseq.get()).unwrap(),
            )];
            let changed_messages = client
                .fetch_with_modifiers(
                    SequenceSet::try_from("1:*").unwrap(),
                    MacroOrMessageDataItemNames::Macro(Macro::Fast),
                    modifiers,
                )
                .await
                .unwrap();

            println!(
                "FETCH CHANGEDSINCE found {} changed messages",
                changed_messages.len()
            );

            // Test SEARCH MODSEQ
            println!("\n--- Testing SEARCH MODSEQ {} ---", test_modseq);

            let search_results = client
                .search(Vec1::from(SearchKey::ModSequence {
                    entry: None,
                    modseq: test_modseq.get(),
                }))
                .await
                .unwrap();

            println!("SEARCH MODSEQ found {} messages", search_results.len());
        } else {
            println!("\nCONDSTORE is not active (no HIGHESTMODSEQ returned)");
        }
    }

    #[cfg(not(feature = "condstore"))]
    {
        println!("This example requires the condstore feature");
        println!("Run with: cargo run --features condstore --example=condstore");
    }
}
