<script setup lang="ts">
import { useAppStore } from '@/stores/app';
import Button from '@/components/ui/Button.vue';
import { Moon, Sun, Menu, X } from 'lucide-vue-next';

const store = useAppStore();
</script>

<template>
  <header class="sticky top-0 z-50 w-full glass border-b">
    <div class="container flex h-16 items-center justify-between px-4 mx-auto">
      <!-- Logo -->
      <a href="/" class="flex items-center space-x-2">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-apple-blue to-apple-teal flex items-center justify-center">
          <span class="text-white font-bold text-sm">{{SERVICE_NAME.charAt(0).toUpperCase()}}</span>
        </div>
        <span class="font-semibold text-lg text-foreground">{{SERVICE_NAME}}</span>
      </a>

      <!-- Navigation -->
      <nav class="hidden md:flex items-center space-x-6">
        <a href="/" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
          Home
        </a>
        <a href="/features" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
          Features
        </a>
        <a href="/about" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
          About
        </a>
      </nav>

      <!-- Actions -->
      <div class="flex items-center space-x-2">
        <!-- Theme Toggle -->
        <Button 
          variant="ghost" 
          size="icon"
          @click="store.toggleTheme()"
          aria-label="Toggle theme"
        >
          <Moon v-if="!store.isDark" class="h-5 w-5" />
          <Sun v-else class="h-5 w-5" />
        </Button>

        <!-- Mobile Menu Toggle -->
        <Button 
          variant="ghost" 
          size="icon"
          class="md:hidden"
          @click="store.toggleSidebar()"
          aria-label="Toggle menu"
        >
          <X v-if="store.sidebarOpen" class="h-5 w-5" />
          <Menu v-else class="h-5 w-5" />
        </Button>
      </div>
    </div>
  </header>
</template>
