<template>
  <div class="min-h-screen bg-base-200">
    <nav class="navbar bg-base-100 shadow-lg">
      <div class="flex-1">
        <NuxtLink to="/" class="btn btn-ghost text-xl">
          {{ $config.public.appName || 'App' }}
        </NuxtLink>
      </div>
      <div class="flex-none">
        <ul class="menu menu-horizontal px-1">
          <li><NuxtLink to="/">Home</NuxtLink></li>
          <li><NuxtLink to="/about">About</NuxtLink></li>
        </ul>
      </div>
    </nav>
    
    <main class="container mx-auto p-4">
      <slot />
    </main>
    
    <footer class="footer footer-center p-4 bg-base-300 text-base-content">
      <aside>
        <p>Built with Nuxt 3</p>
      </aside>
    </footer>
  </div>
</template>
