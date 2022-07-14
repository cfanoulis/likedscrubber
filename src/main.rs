//! This example showcases how streams can be used for asynchro&nous automatic
//! pagination.
//!
//! Asynchronous iteration is a bit uglier, since there's currently no
//! syntactic sugar for `for` loops. See this article for more information:
//!
//! https://rust-lang.github.io/async-book/05_streams/02_iteration_and_concurrency.html
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use futures_util::{StreamExt};
use rand::RngCore;
use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Credentials, OAuth, model::{ PlayableId, TrackId}};

#[tokio::main]
async fn main() {
    // You can use any logger for debugging.
    let mut log_builder = pretty_env_logger::formatted_builder();
    log_builder.filter_level(log::LevelFilter::Info);
    log_builder.init();

    // May require the `env-file` feature enabled if the environment variables
    // aren't configured manually.
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-library-read", "user-library-modify", "playlist-modify-public", "playlist-read-private", "playlist-modify-private")).unwrap();

    let mut spotify = AuthCodePkceSpotify::new(creds, oauth);

    // Obtaining the access token
    let url = spotify.get_authorize_url(Some(69)).unwrap();
    // This function requires the `cli` feature enabled.
    spotify.prompt_for_token(&url).await.unwrap();

    let me = spotify.me().await.unwrap();

    println!("\n\n\n\n");
    info!("Hi {}! Let's get your Liked Songs all clean up", &me.display_name.unwrap_or_else(|| "random user".to_string()));
    let old_songs_id = rand::rngs::ThreadRng::default().next_u32() % 24;
    let playlist = spotify.user_playlist_create(&me.id, &format!("Your old songs # {}", old_songs_id.to_string()), Some(false), Some(false), Some("Your old songs, as saved by spoti_clean_liked")).await.unwrap();
    warn!("Playlist made, time to go KABOOM!");
    let stream = spotify.current_user_saved_tracks(None);
    let liked_songs = stream.filter_map(|x| async move {
        match x {
            Ok(track) => Some(track.track.id.unwrap()),
            Err(_) => None
        }
    }).collect::<Vec<TrackId>>().await;

    let chunks = liked_songs.chunks(50);

    for chunk in chunks {
        // TODO: Uff... find a more elegant solution for the ID conversion
        let playable = chunk
            .iter()
            .map(|id| id as &dyn PlayableId)
            .collect::<Vec<&dyn PlayableId>>();

        let playlist_result = spotify.playlist_add_items(&playlist.id,playable, None).await;
        match playlist_result {
            Ok(_) => {
                info!("* Added {} songs to your archive,", chunk.len());
            }
            Err(e) => {
                error!("Failed to add songs to your archive: {}", e);
                error!("Spoticlean will now exit.");
                std::process::exit(1)
            }
        }
        
        let liked_removed_result = spotify.current_user_saved_tracks_delete(chunk).await;
        match liked_removed_result {
            Ok(_) => {
                info!("And waved {} songs goodbye from your likes", chunk.len());
            }
            Err(e) => {
                error!("Failed to remove songs: {}", e);
            }
        }
    }

    info!("Done! Enjoy your clean slate!")
}