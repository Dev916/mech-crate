//! 404 Not Found Page

use leptos::prelude::*;

use crate::app::components::{Button, ButtonVariant, Container, Footer, Header};

/// 404 Not Found page
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Header/>
        <Container>
            <div class="flex flex-col items-center justify-center min-h-[60vh] space-y-6">
                <div class="text-center space-y-2">
                    <h1 class="text-6xl font-bold text-primary">"404"</h1>
                    <h2 class="text-2xl font-semibold">"Page Not Found"</h2>
                    <p class="text-muted-foreground">
                        "The page you're looking for doesn't exist or has been moved."
                    </p>
                </div>
                <a href="/">
                    <Button variant=ButtonVariant::Default>
                        "Back to Home"
                    </Button>
                </a>
            </div>
        </Container>
        <Footer/>
    }
}
