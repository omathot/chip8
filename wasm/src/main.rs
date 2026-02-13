mod app;
use app::App;
use leptos::prelude::*;
use leptos_meta::*;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
	use axum::Router;
	use leptos_axum::{LeptosRoutes, generate_route_list};

	let conf = get_configuration(None).unwrap();
	let leptos_options = conf.leptos_options;
	let addr = leptos_options.site_addr;
	let routes = generate_route_list(App);

	let app = Router::new()
		.leptos_routes(&leptos_options, routes, {
			let options = leptos_options.clone();
			move || shell(options.clone())
		})
		.fallback(leptos_axum::file_and_error_handler(shell))
		.with_state(leptos_options);

	let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
	println!("Listening on http://{}", &addr);
	axum::serve(listener, app.into_make_service())
		.await
		.unwrap();
}

fn shell(options: LeptosOptions) -> impl IntoView {
	view! {
		<!DOCTYPE html>
		<html lang="en">
			<head>
				<meta charset="utf-8"/>
				<meta name="viewport" content="width=device-width, initial-scale=1"/>
				<AutoReload options=options.clone()/>
				<HydrationScripts options=options.clone()/>
				<MetaTags/>
			</head>
			<body>
				<App/>
			</body>
		</html>
	}
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
