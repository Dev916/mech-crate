export interface User {
    id: number;
    name: string;
    email: string;
    email_verified_at?: string;
}

export type PageProps<
    T extends Record<string, unknown> = Record<string, unknown>
> = T & {
    auth: {
        user: User | null;
    };
    flash: {
        success?: string;
        error?: string;
    };
    ziggy: {
        location: string;
        [key: string]: unknown;
    };
};

declare module 'vue' {
    interface ComponentCustomProperties {
        route: typeof import('ziggy-js').route;
    }
}

declare module '@inertiajs/vue3' {
    export function usePage<T = PageProps>(): {
        props: T;
        url: string;
        component: string;
        version: string | null;
    };
}
