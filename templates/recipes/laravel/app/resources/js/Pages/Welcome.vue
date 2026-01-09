<script setup lang="ts">
import { Head, Link } from '@inertiajs/vue3';

defineProps<{
    appName: string;
    laravelVersion: string;
    phpVersion: string;
}>();
</script>

<template>
    <Head title="Welcome" />
    
    <div class="min-h-screen flex flex-col items-center justify-center p-8">
        <div class="max-w-2xl w-full">
            <!-- Logo/Title -->
            <div class="text-center mb-12">
                <h1 class="text-5xl font-bold bg-gradient-to-r from-indigo-500 to-purple-600 bg-clip-text text-transparent mb-4">
                    {{ appName }}
                </h1>
                <p class="text-slate-600 dark:text-slate-400 text-lg">
                    Laravel 12 + Filament + Inertia.js SSR
                </p>
            </div>
            
            <!-- Feature Cards -->
            <div class="grid md:grid-cols-2 gap-6 mb-12">
                <div class="bg-white dark:bg-slate-800 rounded-xl p-6 shadow-sm border border-slate-200 dark:border-slate-700">
                    <div class="w-10 h-10 bg-indigo-100 dark:bg-indigo-900 rounded-lg flex items-center justify-center mb-4">
                        <svg class="w-6 h-6 text-indigo-600 dark:text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2">Octane + Swoole</h3>
                    <p class="text-slate-600 dark:text-slate-400 text-sm">
                        10x faster than PHP-FPM with persistent connections and coroutines.
                    </p>
                </div>
                
                <div class="bg-white dark:bg-slate-800 rounded-xl p-6 shadow-sm border border-slate-200 dark:border-slate-700">
                    <div class="w-10 h-10 bg-amber-100 dark:bg-amber-900 rounded-lg flex items-center justify-center mb-4">
                        <svg class="w-6 h-6 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2">Filament Admin</h3>
                    <p class="text-slate-600 dark:text-slate-400 text-sm">
                        Beautiful admin panel with TALL stack. Access at <code class="text-xs bg-slate-100 dark:bg-slate-700 px-1 rounded">/admin</code>
                    </p>
                </div>
                
                <div class="bg-white dark:bg-slate-800 rounded-xl p-6 shadow-sm border border-slate-200 dark:border-slate-700">
                    <div class="w-10 h-10 bg-emerald-100 dark:bg-emerald-900 rounded-lg flex items-center justify-center mb-4">
                        <svg class="w-6 h-6 text-emerald-600 dark:text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
                        </svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2">Inertia.js SSR</h3>
                    <p class="text-slate-600 dark:text-slate-400 text-sm">
                        Vue 3 SPA experience with server-side rendering for SEO and performance.
                    </p>
                </div>
                
                <div class="bg-white dark:bg-slate-800 rounded-xl p-6 shadow-sm border border-slate-200 dark:border-slate-700">
                    <div class="w-10 h-10 bg-rose-100 dark:bg-rose-900 rounded-lg flex items-center justify-center mb-4">
                        <svg class="w-6 h-6 text-rose-600 dark:text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                        </svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2">Docker Ready</h3>
                    <p class="text-slate-600 dark:text-slate-400 text-sm">
                        Production-ready Docker setup with Traefik routing.
                    </p>
                </div>
            </div>
            
            <!-- Version Info -->
            <div class="text-center text-sm text-slate-500 dark:text-slate-500">
                Laravel v{{ laravelVersion }} • PHP v{{ phpVersion }}
            </div>
        </div>
    </div>
</template>
