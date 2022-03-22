pub mod model;

use yew::prelude::*;
use yew_router::prelude::*;
use reqwasm::http::Request;
 



#[derive(Clone, Routable, PartialEq)]
enum Route {
	#[at("/")]
	Home,
	#[at("/guilds")]
	Guilds, 
	#[not_found]
	#[at("/404")]
	NotFound,
}
enum Msg {
    AddOne,
}

struct Model {
    value: i64,
}

fn switch(routes: &Route) -> Html {
	match routes {
		Route::Home => html! { <h1>{ "Home" }</h1> },
		Route::Guilds => html! { <Guilds />},
		Route::NotFound => html! { <NotFound /> },
	}
}

#[function_component(Guilds)]
fn guilds() -> Html {
	let guilds: crate::model::guilds::Guilds;
	let guilds = use_state(|| crate::model::guilds::Guilds { guilds: vec![] });
	{
		let guilds = guilds.clone();
		use_effect_with_deps(move |_| {
			let guilds = guilds;
			wasm_bindgen_futures::spawn_local(async move {
				let f_guilds: crate::model::guilds::Guilds = Request::get("http://localhost:8081/api/guilds")
					.send()
					.await
					.unwrap()
					.json()
					.await
					.unwrap();
				guilds.set(f_guilds);
			});
			|| ()
		}, ())
	}
	html! {
		<div id="guilds">
			{
				(*guilds).clone().guilds.into_iter().map(|guild| html! {
					<div key={guild.id}>{ format!("Guild {}, contains {} members", guild.name, guild.member_count)}</div>
				}).collect::<Html>()
			}
		</div>
	}
}

#[function_component(NotFound)]
fn notfound() -> Html {
	html! {
		<h1> {"Not Found" } </h1>
	}
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            value: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
			<BrowserRouter>
           		<Switch<Route> render={Switch::render(switch)} />
        	</BrowserRouter>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}