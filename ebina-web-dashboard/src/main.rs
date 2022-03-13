use yew::prelude::*;
use yew_router::prelude::*;


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
		Route::Guilds => todo!(),
		Route::NotFound => html! { <h1>{ "404" }</h1> },
	}
}

fn notfound() -> Html {
	html! {
		
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