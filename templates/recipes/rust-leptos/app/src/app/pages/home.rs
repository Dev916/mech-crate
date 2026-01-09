//! Home Page

use leptos::prelude::*;

use crate::app::components::{
    Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle,
    Container, Footer, Header,
};

/// Home page component
#[component]
pub fn HomePage() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <Header/>
        <Container>
            <div class="flex flex-col items-center justify-center min-h-[60vh] space-y-8">
                // Hero section
                <div class="text-center space-y-4">
                    <h1 class="text-4xl font-extrabold tracking-tight lg:text-5xl">
                        "Welcome to "
                        <span class="text-primary">"{{SERVICE_NAME}}"</span>
                    </h1>
                    <p class="max-w-[42rem] mx-auto text-muted-foreground sm:text-xl">
                        "A full-stack Rust application built with Leptos SSR, Actix-web, "
                        "and shadcn-ui components."
                    </p>
                </div>

                // Interactive demo card
                <Card class="w-full max-w-md">
                    <CardHeader>
                        <CardTitle>"Interactive Counter"</CardTitle>
                        <CardDescription>
                            "This counter is reactive and works with SSR hydration."
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div class="flex items-center justify-center gap-4">
                            <Button
                                variant=ButtonVariant::Outline
                                on:click=move |_| set_count.update(|n| *n -= 1)
                            >
                                "-"
                            </Button>
                            <span class="text-4xl font-bold tabular-nums min-w-[3ch] text-center">
                                {count}
                            </span>
                            <Button on:click=move |_| set_count.update(|n| *n += 1)>
                                "+"
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                // Feature cards
                <div class="grid gap-6 md:grid-cols-3 w-full max-w-4xl">
                    <FeatureCard
                        title="Leptos SSR"
                        description="Server-side rendering with hydration for fast initial loads and SEO."
                        icon="🚀"
                    />
                    <FeatureCard
                        title="Actor Model"
                        description="Actix actors for stateful entities with message-based concurrency."
                        icon="🎭"
                    />
                    <FeatureCard
                        title="Type-Safe SQL"
                        description="SQLx for compile-time checked queries with PostgreSQL."
                        icon="🗄️"
                    />
                </div>

                // CTA buttons
                <div class="flex gap-4">
                    <Button>
                        "Get Started"
                    </Button>
                    <Button variant=ButtonVariant::Outline>
                        "Documentation"
                    </Button>
                </div>
            </div>
        </Container>
        <Footer/>
    }
}

#[component]
fn FeatureCard(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] icon: String,
) -> impl IntoView {
    view! {
        <Card>
            <CardHeader>
                <div class="text-4xl mb-2">{icon}</div>
                <CardTitle class="text-lg">{title}</CardTitle>
            </CardHeader>
            <CardContent>
                <p class="text-sm text-muted-foreground">{description}</p>
            </CardContent>
        </Card>
    }
}
