//! Leptos Application Components
//!
//! This module contains:
//! - `App` - Root component with router
//! - `components/` - Reusable UI components (shadcn-ui based)
//! - `pages/` - Page components for each route

pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use pages::{HomePage, NotFoundPage};

/// Root Application Component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/{{SERVICE_SLUG}}.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta name="description" content="{{SERVICE_NAME}} - Built with Leptos"/>

        <Title text="{{SERVICE_NAME}}"/>

        <Router>
            <main class="min-h-screen bg-background text-foreground antialiased">
                <Routes fallback=|| NotFoundPage>
                    <Route path=path!("/") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}
