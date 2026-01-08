# Apple-Inspired UI/UX Design Guidelines for LLMs

## Core Philosophy
**"Simplicity is the ultimate sophistication"** - This document provides comprehensive guidelines for implementing Apple's design principles in web applications. Every UI element should embody clarity, deference, and depth while maintaining exceptional performance.

---

## 🎯 Fundamental Principles

### 1. **Clarity**
- Content is king - let it breathe with generous whitespace
- Typography must be instantly readable
- Icons and images should communicate meaning without explanation
- Every element serves a purpose - remove anything decorative without function

### 2. **Deference**
- The UI should never compete with content
- Subtle, understated interface elements
- Fluid motion that feels natural, never jarring
- Transparency and blur to maintain context

### 3. **Depth**
- Use layers and realistic motion to convey hierarchy
- Shadows should be soft and multi-layered
- Elements should feel tactile and responsive
- Z-axis positioning creates understanding

---

## 🎨 Visual Design Language

### Color Palette

```css
/* Primary Apple-Inspired Colors */
:root {
  /* Blues (Primary Actions) */
  --apple-blue: #007AFF;
  --apple-blue-hover: #0051D5;
  --apple-blue-active: #004CCA;
  
  /* System Grays */
  --apple-gray-1: #8E8E93;
  --apple-gray-2: #C7C7CC;
  --apple-gray-3: #D1D1D6;
  --apple-gray-4: #E5E5EA;
  --apple-gray-5: #F2F2F7;
  --apple-gray-6: #FFFFFF;
  
  /* Semantic Colors */
  --apple-green: #34C759;
  --apple-red: #FF3B30;
  --apple-orange: #FF9500;
  --apple-yellow: #FFCC00;
  --apple-purple: #AF52DE;
  --apple-pink: #FF2D55;
  
  /* Dark Mode Support */
  --apple-background: #FFFFFF;
  --apple-background-elevated: #F2F2F7;
  --apple-text-primary: #000000;
  --apple-text-secondary: #3C3C43;
  --apple-text-tertiary: #C7C7CC;
}

@media (prefers-color-scheme: dark) {
  :root {
    --apple-background: #000000;
    --apple-background-elevated: #1C1C1E;
    --apple-text-primary: #FFFFFF;
    --apple-text-secondary: #EBEBF5;
    --apple-text-tertiary: #48484A;
  }
}
```

### Typography

```css
/* Typography System */
.apple-typography {
  /* Use SF Pro or fallback to system fonts */
  --font-stack: -apple-system, BlinkMacSystemFont, "SF Pro Display", 
                "SF Pro Text", "Helvetica Neue", Helvetica, Arial, sans-serif;
  
  /* Type Scale */
  --text-xs: 11px;     /* Caption 2 */
  --text-sm: 12px;     /* Caption 1 */
  --text-base: 15px;   /* Footnote */
  --text-body: 17px;   /* Body */
  --text-lg: 20px;     /* Title 3 */
  --text-xl: 22px;     /* Title 2 */
  --text-2xl: 28px;    /* Title 1 */
  --text-3xl: 34px;    /* Large Title */
  
  /* Font Weights */
  --font-regular: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
  
  /* Line Heights */
  --leading-tight: 1.2;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;
  
  /* Letter Spacing */
  --tracking-tight: -0.02em;
  --tracking-normal: 0;
  --tracking-wide: 0.01em;
}

/* Usage Example */
.heading-1 {
  font-size: var(--text-3xl);
  font-weight: var(--font-bold);
  line-height: var(--leading-tight);
  letter-spacing: var(--tracking-tight);
}
```

### Spacing System

```css
/* Consistent Spacing Scale */
:root {
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
  --space-3xl: 64px;
  --space-4xl: 96px;
  
  /* Component-specific spacing */
  --padding-button: 12px 24px;
  --padding-card: 20px;
  --padding-section: 40px;
  --gap-elements: 16px;
}
```

---

## 🔲 Glassmorphism & Material Effects

### Glass Effect Implementation

```css
/* Glassmorphism Mixin */
.glass-effect {
  /* Background with transparency */
  background: rgba(255, 255, 255, 0.7);
  
  /* Backdrop blur for the glass effect */
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  
  /* Subtle border for definition */
  border: 1px solid rgba(255, 255, 255, 0.18);
  
  /* Multi-layered shadows for depth */
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.5),
    inset 0 -1px 0 rgba(0, 0, 0, 0.05);
}

/* Dark mode glass */
@media (prefers-color-scheme: dark) {
  .glass-effect {
    background: rgba(30, 30, 30, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
}
```

### Shadow System

```css
/* Elevation Shadows */
:root {
  /* Subtle elevation */
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.04),
               0 1px 2px rgba(0, 0, 0, 0.08);
  
  /* Medium elevation */
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.04),
               0 2px 4px rgba(0, 0, 0, 0.08);
  
  /* High elevation */
  --shadow-lg: 0 10px 25px rgba(0, 0, 0, 0.05),
               0 6px 10px rgba(0, 0, 0, 0.08);
  
  /* Extra high elevation */
  --shadow-xl: 0 20px 40px rgba(0, 0, 0, 0.08),
               0 10px 20px rgba(0, 0, 0, 0.12);
  
  /* Colored shadows for buttons */
  --shadow-blue: 0 4px 14px rgba(0, 122, 255, 0.3);
  --shadow-green: 0 4px 14px rgba(52, 199, 89, 0.3);
  --shadow-red: 0 4px 14px rgba(255, 59, 48, 0.3);
}
```

---

## 🎛 Component Patterns

### Buttons

```css
/* Apple-style Button */
.btn-apple {
  /* Structure */
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 10px 20px;
  min-width: 64px;
  
  /* Typography */
  font-family: var(--font-stack);
  font-size: 15px;
  font-weight: 500;
  letter-spacing: 0.01em;
  
  /* Appearance */
  color: white;
  background: var(--apple-blue);
  border: none;
  border-radius: 12px;
  
  /* Effects */
  box-shadow: var(--shadow-md), var(--shadow-blue);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  
  /* Interaction */
  cursor: pointer;
  user-select: none;
  -webkit-tap-highlight-color: transparent;
}

.btn-apple:hover {
  background: var(--apple-blue-hover);
  transform: translateY(-1px) scale(1.02);
  box-shadow: var(--shadow-lg), var(--shadow-blue);
}

.btn-apple:active {
  background: var(--apple-blue-active);
  transform: translateY(0) scale(0.98);
  box-shadow: var(--shadow-sm);
}

/* Glass variant */
.btn-glass {
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: var(--apple-text-primary);
}

/* Size variants */
.btn-small { 
  padding: 6px 14px; 
  font-size: 13px; 
  border-radius: 10px;
}

.btn-large { 
  padding: 14px 28px; 
  font-size: 17px; 
  border-radius: 14px;
}
```

### Cards

```css
/* Apple-style Card */
.card-apple {
  /* Structure */
  position: relative;
  padding: var(--padding-card);
  
  /* Appearance */
  background: var(--apple-background-elevated);
  border-radius: 16px;
  
  /* Effects */
  box-shadow: var(--shadow-md);
  transition: all 0.3s var(--ease-smooth);
}

.card-apple:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-xl);
}

/* Glass card variant */
.card-glass {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
  border: 1px solid rgba(255, 255, 255, 0.18);
}
```

### Forms

```css
/* Apple-style Input */
.input-apple {
  /* Structure */
  width: 100%;
  padding: 12px 16px;
  
  /* Typography */
  font-family: var(--font-stack);
  font-size: 17px;
  
  /* Appearance */
  background: var(--apple-gray-5);
  border: 1px solid transparent;
  border-radius: 10px;
  
  /* Effects */
  transition: all 0.2s var(--ease-smooth);
}

.input-apple:focus {
  outline: none;
  background: white;
  border-color: var(--apple-blue);
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.1);
}

/* Search input with icon */
.search-apple {
  position: relative;
}

.search-apple::before {
  content: "🔍";
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.5;
}

.search-apple input {
  padding-left: 36px;
}
```

### Navigation

```css
/* Apple-style Navigation Bar */
.navbar-apple {
  /* Structure */
  position: fixed;
  top: 0;
  width: 100%;
  height: 44px;
  z-index: 1000;
  
  /* Glass effect */
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: saturate(180%) blur(20px);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
  
  /* Border */
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

/* Tab bar */
.tabbar-apple {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--apple-gray-5);
  border-radius: 10px;
}

.tab-apple {
  flex: 1;
  padding: 8px 16px;
  background: transparent;
  border: none;
  border-radius: 8px;
  transition: all 0.2s;
}

.tab-apple.active {
  background: white;
  box-shadow: var(--shadow-sm);
}
```

---

## 🎬 Animations & Interactions

### Motion Principles

```css
/* Animation Timing Functions */
:root {
  /* Apple's preferred easing curves */
  --ease-in-out-quart: cubic-bezier(0.77, 0, 0.175, 1);
  --ease-out-expo: cubic-bezier(0.19, 1, 0.22, 1);
  --ease-spring: cubic-bezier(0.68, -0.55, 0.265, 1.55);
  
  /* Duration scale */
  --duration-instant: 100ms;
  --duration-fast: 200ms;
  --duration-normal: 300ms;
  --duration-slow: 500ms;
  --duration-slower: 700ms;
}

/* Smooth transitions for all interactive elements */
* {
  transition-property: transform, opacity, box-shadow;
  transition-duration: var(--duration-fast);
  transition-timing-function: var(--ease-in-out-quart);
}
```

### Micro-interactions

```css
/* Ripple effect */
@keyframes ripple {
  0% {
    transform: scale(0);
    opacity: 1;
  }
  100% {
    transform: scale(4);
    opacity: 0;
  }
}

.ripple-container {
  position: relative;
  overflow: hidden;
}

.ripple-container::after {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.5);
  transform: translate(-50%, -50%);
  animation: ripple 0.6s ease-out;
}

/* Pulse animation for loading states */
@keyframes pulse-subtle {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

.loading {
  animation: pulse-subtle 2s ease-in-out infinite;
}

/* Smooth appear animation */
@keyframes fade-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-in {
  animation: fade-up 0.5s var(--ease-out-expo);
}
```

---

## 📱 Responsive Design

### Breakpoints

```css
/* Apple-inspired breakpoints */
:root {
  --screen-xs: 320px;   /* iPhone SE */
  --screen-sm: 375px;   /* iPhone 12 mini */
  --screen-md: 390px;   /* iPhone 14 */
  --screen-lg: 768px;   /* iPad mini */
  --screen-xl: 1024px;  /* iPad Pro 11" */
  --screen-2xl: 1366px; /* iPad Pro 12.9" */
  --screen-3xl: 1920px; /* Desktop */
}

/* Media query mixins */
@media (min-width: 768px) {
  /* Tablet and up */
}

@media (min-width: 1024px) {
  /* Desktop and up */
}

@media (hover: hover) {
  /* Only for devices with hover capability */
}

@media (prefers-reduced-motion: reduce) {
  /* Respect user's motion preferences */
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## 🛠 Implementation Checklist

When implementing any UI component, ensure it meets these criteria:

### Visual Polish
- [ ] Uses consistent spacing from the design system
- [ ] Implements proper typography hierarchy
- [ ] Has appropriate shadows for elevation
- [ ] Includes subtle borders where needed
- [ ] Uses the correct border radius (8px, 10px, 12px, 16px)

### Interaction Design
- [ ] Has hover states for all interactive elements
- [ ] Includes focus states for accessibility
- [ ] Implements smooth transitions (200-300ms)
- [ ] Provides visual feedback for actions
- [ ] Supports keyboard navigation

### Performance
- [ ] Uses CSS transforms for animations (not position/size)
- [ ] Implements will-change for animated properties
- [ ] Lazy loads images and heavy content
- [ ] Minimizes repaints and reflows
- [ ] Uses CSS containment where appropriate

### Accessibility
- [ ] Maintains WCAG AA contrast ratios (4.5:1 for normal text)
- [ ] Includes proper ARIA labels
- [ ] Supports keyboard navigation
- [ ] Respects prefers-reduced-motion
- [ ] Has visible focus indicators

### Responsive Design
- [ ] Works on all screen sizes
- [ ] Touch targets are at least 44x44px
- [ ] Text remains readable without zooming
- [ ] Images are optimized for retina displays
- [ ] Layout adapts gracefully

---

## 💡 Best Practices for LLMs

### When Creating New Components

1. **Start with Structure**: Define the HTML semantics first
2. **Apply Base Styles**: Use the design tokens (colors, spacing, typography)
3. **Add Glass Effects**: Apply glassmorphism where appropriate
4. **Implement States**: Add hover, active, focus, and disabled states
5. **Polish with Animation**: Add subtle transitions and micro-interactions
6. **Test Responsiveness**: Ensure it works across all breakpoints

### Code Organization

```css
/* Component structure example */
.component {
  /* 1. Layout */
  display: flex;
  position: relative;
  
  /* 2. Sizing */
  width: 100%;
  padding: var(--space-md);
  
  /* 3. Typography */
  font-family: var(--font-stack);
  font-size: var(--text-base);
  
  /* 4. Colors & Backgrounds */
  color: var(--apple-text-primary);
  background: var(--apple-background);
  
  /* 5. Borders & Shadows */
  border: 1px solid var(--apple-gray-4);
  border-radius: 12px;
  box-shadow: var(--shadow-md);
  
  /* 6. Effects & Animations */
  transition: all 0.2s var(--ease-smooth);
  
  /* 7. Interaction */
  cursor: pointer;
  user-select: none;
}
```

### Common Patterns to Follow

```scss
/* ALWAYS use these patterns */

// Buttons should lift on hover
.button:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}

// Cards should have subtle shadows
.card {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

// Inputs should have focus rings
.input:focus {
  outline: none;
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.1);
}

// Modals should have glass backgrounds
.modal-backdrop {
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(8px);
}

// Loading states should pulse
.skeleton {
  animation: pulse-subtle 2s infinite;
}
```

### Anti-patterns to Avoid

```scss
/* NEVER do these */

// ❌ Harsh shadows
box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);

// ❌ Pure black/white
color: #000000;
background: #FFFFFF;

// ❌ Sharp corners on interactive elements
border-radius: 0;

// ❌ Instant transitions
transition: none;

// ❌ Thick borders
border: 3px solid;

// ❌ Aggressive animations
animation: bounce 0.5s infinite;
```

---

## 📋 Quick Reference

### CSS Variables to Use

```css
/* Copy this into your project */
:root {
  /* Core Colors */
  --apple-blue: #007AFF;
  --apple-gray: #8E8E93;
  --apple-background: #FFFFFF;
  
  /* Glass Effects */
  --glass-bg: rgba(255, 255, 255, 0.7);
  --glass-blur: blur(20px);
  --glass-border: rgba(255, 255, 255, 0.18);
  
  /* Shadows */
  --shadow: 0 4px 14px rgba(0, 0, 0, 0.08);
  
  /* Animation */
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
  --duration: 200ms;
  
  /* Spacing */
  --space: 16px;
  --radius: 12px;
}
```

### Component Template

```html
<!-- Basic Apple-style component -->
<div class="component-apple">
  <h3 class="title-apple">Title</h3>
  <p class="text-apple">Content goes here</p>
  <button class="btn-apple">Action</button>
</div>
```

```css
.component-apple {
  padding: 20px;
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  transition: all var(--duration) var(--ease);
}
```

---

## 🎯 Summary

When implementing Apple-inspired UI:

1. **Prioritize Clarity**: Every element should have a clear purpose
2. **Embrace Whitespace**: Don't fear empty space - it creates focus
3. **Layer with Purpose**: Use depth to establish hierarchy
4. **Animate Thoughtfully**: Every motion should feel natural
5. **Stay Consistent**: Use the design system religiously
6. **Polish Relentlessly**: The details make the difference

Remember: **Great design is invisible**. Users should focus on their tasks, not the interface.

---

*Last Updated: 2024*
*Version: 1.0*
