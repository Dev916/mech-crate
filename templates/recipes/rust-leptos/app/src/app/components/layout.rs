//! Layout Components

use leptos::prelude::*;

/// Main navigation header
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <div class="container flex h-14 max-w-screen-2xl items-center">
                <div class="mr-4 flex">
                    <a href="/" class="mr-6 flex items-center space-x-2">
                        <span class="font-bold text-xl">
                            "{{SERVICE_NAME}}"
                        </span>
                    </a>
                </div>
                <nav class="flex items-center gap-4 text-sm lg:gap-6">
                    <a href="/" class="transition-colors hover:text-foreground/80 text-foreground/60">
                        "Home"
                    </a>
                    <a href="/about" class="transition-colors hover:text-foreground/80 text-foreground/60">
                        "About"
                    </a>
                </nav>
                <div class="flex flex-1 items-center justify-end space-x-2">
                    <ThemeToggle/>
                </div>
            </div>
        </header>
    }
}

/// Theme toggle button (light/dark mode)
#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (dark_mode, set_dark_mode) = signal(false);

    let toggle_theme = move |_| {
        set_dark_mode.update(|v| *v = !*v);
        // In real app, would also update document class
    };

    view! {
        <button
            on:click=toggle_theme
            class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-10 w-10"
        >
            <span class="sr-only">"Toggle theme"</span>
            {move || if dark_mode.get() {
                view! { <SunIcon/> }.into_any()
            } else {
                view! { <MoonIcon/> }.into_any()
            }}
        </button>
    }
}

#[component]
fn SunIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="5"/>
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
        </svg>
    }
}

#[component]
fn MoonIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
    }
}

/// Page container with standard padding
#[component]
pub fn Container(children: Children) -> impl IntoView {
    view! {
        <div class="container mx-auto px-4 py-8 max-w-screen-2xl">
            {children()}
        </div>
    }
}

/// Footer component
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-border/40 py-6 md:py-0">
            <div class="container flex flex-col items-center justify-between gap-4 md:h-14 md:flex-row">
                <p class="text-center text-sm leading-loose text-muted-foreground md:text-left">
                    "Built with "
                    <a href="https://leptos.dev" target="_blank" class="font-medium underline underline-offset-4">
                        "Leptos"
                    </a>
                    " + "
                    <a href="https://actix.rs" target="_blank" class="font-medium underline underline-offset-4">
                        "Actix"
                    </a>
                </p>
            </div>
        </footer>
    }
}
