# LLM Build Instructions

## Core Stack Requirements
When generating any frontend code, **you must use the project’s established stack**:

- **Astro** for page routing, islands architecture, and project structure.
- **Vue 3** (script setup + Composition API) as the primary component layer.
- **shadcn-vue** for UI primitives, dialog, dropdowns, forms, buttons, sheets, toasts, typography.
- **PrimeVue** for complex and advanced UI components such as tables, data views, charts, pickers, menus, autocomplete, and advanced interactions.
- **Tailwind CSS** for utility-first styling, spacing, typography, layout, responsive behavior, and design consistency.

Do **not** build custom UI elements when a component already exists in **shadcn-vue** or **PrimeVue** unless explicitly instructed.

## Component Usage Rules

### Prefer shadcn-vue First
Use **shadcn-vue** for:
- Buttons
- Inputs
- Forms
- Dialog / Modal
- Popover
- Dropdown
- Sheet
- Toast
- Avatar
- Badge
- Skeleton
- Navigation
- Cards
- Layout primitives

### Use PrimeVue for Anything Complex
Use **PrimeVue** for:
- DataTable / DataGrid
- Tree / TreeTable
- Charts
- Menus / MegaMenu
- DatePicker / Calendar
- AutoComplete
- MultiSelect
- FileUpload
- Stepper
- SplitButton
- DataView / VirtualScroller

## Bilingual Text Requirements

### Always Use BilingualText for Dual-Language Labels
This app supports Japanese and English bilingual UI. **Never hardcode single-language text** where bilingual display is needed.

Use `BilingualText.vue` (in Vue components) or `BilingualText.astro` (in Astro pages) for:
- Button labels
- Navigation items
- Headings and section titles
- Tab labels
- Menu items
- Form labels
- Any user-facing text that requires both languages

### BilingualText Props
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `en` | string | required | English text |
| `ja` | string | required | Japanese text |
| `layout` | 'stack' \| 'inline' | 'stack' | Column (stack) or row (inline) flexbox |
| `order` | 'en-first' \| 'ja-first' | 'en-first' | Which language appears first |
| `align` | 'start' \| 'center' \| 'end' | 'center' | Flexbox alignment |
| `gap` | string | 'gap-0.5' | Tailwind gap class |
| `enClass` | string | 'text-sm font-semibold tracking-[0.2em]' | English text styling |
| `jaClass` | string | 'text-xs font-medium text-white/80 font-japanese' | Japanese text styling |

### Usage Patterns

**Buttons (centered column layout):**
```vue
<Button>
  <BilingualText 
    en="CONNECT WALLET" 
    ja="ウォレット接続" 
    layout="stack" 
    align="center" 
  />
</Button>
```

**Navigation items (inline row layout):**
```vue
<BilingualText 
  en="Dashboard" 
  ja="ダッシュボード" 
  layout="inline" 
  gap="gap-2" 
/>
```

**Japanese-first context (e.g., /jp/ routes):**
```vue
<BilingualText 
  en="Settings" 
  ja="設定" 
  order="ja-first" 
/>
```

### Important Rules
1. **Never skip BilingualText** — all user-facing labels must support both languages
2. **Use `layout="stack"` for buttons** — vertical stacking with centered alignment
3. **Use `layout="inline"` for navigation/menus** — horizontal display
4. **Swap `order` based on locale** — use 'ja-first' for Japanese-priority pages
5. **Customize classes as needed** — override `enClass`/`jaClass` for context-specific styling

---

## Architecture & Coding Rules

### Always Generate Vue 3 `<script setup>` Components
```
<script setup lang="ts">
</script>

<template>
</template>
```

### Follow Astro Conventions
- Use `src/pages/*.astro` for routing.
- Use `components/*.vue` for interactive islands.
- Use `client:load`, `client:idle`, or `client:visible` as needed.

### Tailwind-First Styling
- Avoid custom CSS unless required.
- Prefer Tailwind for layout, spacing, typography, responsive behavior.
- Keep styling consistent: `p-4`, `gap-6`, `rounded-lg`.

### Never Rebuild What Already Exists
Always ask:  
**"Does this already exist in shadcn-vue or PrimeVue?"**  
If yes, use it.

**"Does this text need to be bilingual?"**  
If yes, use `BilingualText`.

### Composables > Utils
Use Vue composables for shared logic:
- `useAuth()`
- `useApi()`
- `useFormHandler()`

## Interaction Rules

### Use Shadcn Form + VeeValidate or Zod (If Needed)

### Use PrimeVue for Data Presentation

### Use shadcn-vue Toasts for Notifications

## Output Style Expectations
- Full imports for shadcn-vue and PrimeVue.
- Tailwind classes inline.
- Modular logic.
- UX patterns clean and consistent.

## Good Example
- Uses shadcn button + PrimeVue DataTable
- Vue 3 script setup
- Tailwind layout
- Astro wrapper for the page
- BilingualText for all dual-language labels
- Proper `order` prop based on locale context

## Bad Example
- Rebuilding buttons manually
- Custom modal
- Options API
- Custom tables instead of PrimeVue
- Excessive CSS
- Hardcoded single-language text where bilingual is needed
- Manual flexbox for bilingual labels instead of BilingualText
- Forgetting Japanese translations

## Final Behavior
- Faster builds
- Consistent UI
- Fully leverage existing frameworks
- Maintainable architecture
- Full bilingual support via BilingualText — never skip it
