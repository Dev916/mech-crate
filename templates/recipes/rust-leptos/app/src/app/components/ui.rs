//! UI Components (shadcn-ui wrappers)
//!
//! Re-exports and customizes leptos-shadcn-ui components.

use leptos::prelude::*;

/// Primary button component
#[component]
pub fn Button(
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] size: Option<ButtonSize>,
    #[prop(into, optional)] class: Option<String>,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Default);
    let size = size.unwrap_or(ButtonSize::Default);
    
    let base_classes = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";
    
    let variant_classes = match variant {
        ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Destructive => "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        ButtonVariant::Outline => "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Link => "text-primary underline-offset-4 hover:underline",
    };
    
    let size_classes = match size {
        ButtonSize::Default => "h-10 px-4 py-2",
        ButtonSize::Sm => "h-9 rounded-md px-3",
        ButtonSize::Lg => "h-11 rounded-md px-8",
        ButtonSize::Icon => "h-10 w-10",
    };

    let all_classes = format!(
        "{} {} {} {}",
        base_classes,
        variant_classes,
        size_classes,
        class.unwrap_or_default()
    );

    view! {
        <button class=all_classes disabled=disabled>
            {children()}
        </button>
    }
}

#[derive(Clone, Copy, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

#[derive(Clone, Copy, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

/// Card component
#[component]
pub fn Card(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "rounded-lg border bg-card text-card-foreground shadow-sm {}",
        class.unwrap_or_default()
    );
    
    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

#[component]
pub fn CardHeader(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "flex flex-col space-y-1.5 p-6 {}",
        class.unwrap_or_default()
    );
    
    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

#[component]
pub fn CardTitle(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "text-2xl font-semibold leading-none tracking-tight {}",
        class.unwrap_or_default()
    );
    
    view! {
        <h3 class=classes>
            {children()}
        </h3>
    }
}

#[component]
pub fn CardDescription(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "text-sm text-muted-foreground {}",
        class.unwrap_or_default()
    );
    
    view! {
        <p class=classes>
            {children()}
        </p>
    }
}

#[component]
pub fn CardContent(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!("p-6 pt-0 {}", class.unwrap_or_default());
    
    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

#[component]
pub fn CardFooter(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "flex items-center p-6 pt-0 {}",
        class.unwrap_or_default()
    );
    
    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

/// Input component
#[component]
pub fn Input(
    #[prop(into, optional)] class: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] r#type: Option<String>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let classes = format!(
        "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {}",
        class.unwrap_or_default()
    );
    
    view! {
        <input
            type=r#type.unwrap_or_else(|| "text".to_string())
            class=classes
            placeholder=placeholder.unwrap_or_default()
            disabled=disabled
        />
    }
}

/// Badge component
#[component]
pub fn Badge(
    #[prop(into, optional)] variant: Option<BadgeVariant>,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(BadgeVariant::Default);
    
    let base_classes = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";
    
    let variant_classes = match variant {
        BadgeVariant::Default => "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
        BadgeVariant::Secondary => "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        BadgeVariant::Destructive => "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
        BadgeVariant::Outline => "text-foreground",
    };
    
    let all_classes = format!(
        "{} {} {}",
        base_classes,
        variant_classes,
        class.unwrap_or_default()
    );

    view! {
        <div class=all_classes>
            {children()}
        </div>
    }
}

#[derive(Clone, Copy, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

/// Alert component
#[component]
pub fn Alert(
    #[prop(into, optional)] variant: Option<AlertVariant>,
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(AlertVariant::Default);
    
    let base_classes = "relative w-full rounded-lg border p-4";
    
    let variant_classes = match variant {
        AlertVariant::Default => "bg-background text-foreground",
        AlertVariant::Destructive => "border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive",
    };
    
    let all_classes = format!(
        "{} {} {}",
        base_classes,
        variant_classes,
        class.unwrap_or_default()
    );

    view! {
        <div role="alert" class=all_classes>
            {children()}
        </div>
    }
}

#[derive(Clone, Copy, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

#[component]
pub fn AlertTitle(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "mb-1 font-medium leading-none tracking-tight {}",
        class.unwrap_or_default()
    );
    
    view! {
        <h5 class=classes>
            {children()}
        </h5>
    }
}

#[component]
pub fn AlertDescription(
    #[prop(into, optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "text-sm [&_p]:leading-relaxed {}",
        class.unwrap_or_default()
    );
    
    view! {
        <div class=classes>
            {children()}
        </div>
    }
}
