<script setup lang="ts">
const currentYear = new Date().getFullYear();
</script>

<template>
  <footer class="border-t bg-muted/30">
    <div class="container mx-auto px-4 py-12">
      <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
        <!-- Brand -->
        <div class="col-span-1 md:col-span-2">
          <a href="/" class="flex items-center space-x-2 mb-4">
            <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-apple-blue to-apple-teal flex items-center justify-center">
              <span class="text-white font-bold text-sm">{{SERVICE_NAME.charAt(0).toUpperCase()}}</span>
            </div>
            <span class="font-semibold text-lg text-foreground">{{SERVICE_NAME}}</span>
          </a>
          <p class="text-subhead text-muted-foreground max-w-md">
            Built with Astro + Vue 3, designed with Apple-inspired aesthetics.
            Fast, modern, and delightful.
          </p>
        </div>

        <!-- Links -->
        <div>
          <h4 class="font-semibold text-foreground mb-4">Product</h4>
          <ul class="space-y-2">
            <li>
              <a href="/features" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
                Features
              </a>
            </li>
            <li>
              <a href="/pricing" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
                Pricing
              </a>
            </li>
            <li>
              <a href="/docs" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
                Documentation
              </a>
            </li>
          </ul>
        </div>

        <!-- More Links -->
        <div>
          <h4 class="font-semibold text-foreground mb-4">Company</h4>
          <ul class="space-y-2">
            <li>
              <a href="/about" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
                About
              </a>
            </li>
            <li>
              <a href="/blog" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
                Blog
              </a>
            </li>
            <li>
              <a href="/contact" class="text-subhead text-muted-foreground hover:text-foreground transition-colors">
                Contact
              </a>
            </li>
          </ul>
        </div>
      </div>

      <!-- Bottom -->
      <div class="border-t mt-12 pt-8 flex flex-col md:flex-row justify-between items-center">
        <p class="text-footnote text-muted-foreground">
          © {{ currentYear }} {{SERVICE_NAME}}. All rights reserved.
        </p>
        <div class="flex space-x-6 mt-4 md:mt-0">
          <a href="/privacy" class="text-footnote text-muted-foreground hover:text-foreground transition-colors">
            Privacy Policy
          </a>
          <a href="/terms" class="text-footnote text-muted-foreground hover:text-foreground transition-colors">
            Terms of Service
          </a>
        </div>
      </div>
    </div>
  </footer>
</template>
