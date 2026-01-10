<script setup lang="ts">
import { Head } from '@inertiajs/vue3'

interface User {
  id: number
  name: string
  email: string
}

defineProps<{
  user: User
}>()
</script>

<template>
  <Head title="Dashboard" />

  <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <!-- Header -->
      <header class="mb-12">
        <h1 class="text-4xl font-bold text-white mb-2">
          Dashboard
        </h1>
        <p class="text-slate-400">
          Welcome back, {{ user.name }}
        </p>
      </header>

      <!-- Stats Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
        <div class="bg-slate-800/50 backdrop-blur-sm rounded-2xl p-6 border border-slate-700/50">
          <div class="flex items-center justify-between mb-4">
            <span class="text-slate-400 text-sm font-medium">Total Users</span>
            <span class="text-emerald-400">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
              </svg>
            </span>
          </div>
          <p class="text-3xl font-bold text-white">1,234</p>
          <p class="text-emerald-400 text-sm mt-1">+12% from last month</p>
        </div>

        <div class="bg-slate-800/50 backdrop-blur-sm rounded-2xl p-6 border border-slate-700/50">
          <div class="flex items-center justify-between mb-4">
            <span class="text-slate-400 text-sm font-medium">Active Sessions</span>
            <span class="text-blue-400">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              </svg>
            </span>
          </div>
          <p class="text-3xl font-bold text-white">842</p>
          <p class="text-blue-400 text-sm mt-1">Currently online</p>
        </div>

        <div class="bg-slate-800/50 backdrop-blur-sm rounded-2xl p-6 border border-slate-700/50">
          <div class="flex items-center justify-between mb-4">
            <span class="text-slate-400 text-sm font-medium">API Requests</span>
            <span class="text-violet-400">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
            </span>
          </div>
          <p class="text-3xl font-bold text-white">45.2K</p>
          <p class="text-violet-400 text-sm mt-1">Last 24 hours</p>
        </div>

        <div class="bg-slate-800/50 backdrop-blur-sm rounded-2xl p-6 border border-slate-700/50">
          <div class="flex items-center justify-between mb-4">
            <span class="text-slate-400 text-sm font-medium">Queue Jobs</span>
            <span class="text-amber-400">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </span>
          </div>
          <p class="text-3xl font-bold text-white">128</p>
          <p class="text-amber-400 text-sm mt-1">Pending jobs</p>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="bg-slate-800/50 backdrop-blur-sm rounded-2xl p-6 border border-slate-700/50">
        <h2 class="text-xl font-semibold text-white mb-6">Quick Actions</h2>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <a
            href="/admin"
            class="flex flex-col items-center justify-center p-6 rounded-xl bg-slate-700/50 hover:bg-slate-700 transition-colors group"
          >
            <svg class="w-8 h-8 text-slate-400 group-hover:text-white mb-3 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <span class="text-slate-300 group-hover:text-white text-sm font-medium transition-colors">Admin Panel</span>
          </a>

          <a
            href="/horizon"
            class="flex flex-col items-center justify-center p-6 rounded-xl bg-slate-700/50 hover:bg-slate-700 transition-colors group"
          >
            <svg class="w-8 h-8 text-slate-400 group-hover:text-white mb-3 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
            <span class="text-slate-300 group-hover:text-white text-sm font-medium transition-colors">Horizon</span>
          </a>

          <button
            class="flex flex-col items-center justify-center p-6 rounded-xl bg-slate-700/50 hover:bg-slate-700 transition-colors group"
          >
            <svg class="w-8 h-8 text-slate-400 group-hover:text-white mb-3 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
            <span class="text-slate-300 group-hover:text-white text-sm font-medium transition-colors">New Item</span>
          </button>

          <button
            class="flex flex-col items-center justify-center p-6 rounded-xl bg-slate-700/50 hover:bg-slate-700 transition-colors group"
          >
            <svg class="w-8 h-8 text-slate-400 group-hover:text-white mb-3 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            <span class="text-slate-300 group-hover:text-white text-sm font-medium transition-colors">Export</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
