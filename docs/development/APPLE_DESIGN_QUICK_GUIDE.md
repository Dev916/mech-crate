# 🍎 Apple Design Quick Implementation Guide for LLMs

## When Asked to Create UI Components

### 1️⃣ **ALWAYS START WITH THIS STRUCTURE**

```css
/* Import Apple utilities if using SCSS */
@import 'path/to/apple-utilities';

/* Or include these CSS variables in your file */
:root {
  --apple-blue: #007AFF;
  --apple-background: #FFFFFF;
  --glass-blur: 20px;
  --shadow: 0 4px 14px rgba(0, 0, 0, 0.08);
  --radius: 12px;
  --ease: cubic-bezier(0.4, 0, 0.2, 1);
}
```

### 2️⃣ **FOR EVERY BUTTON**

```css
.button {
  /* MUST HAVE */
  padding: 10px 20px;
  border-radius: 12px;
  font-weight: 500;
  transition: all 0.2s var(--ease);
  
  /* GLASS EFFECT (if appropriate) */
  background: rgba(0, 122, 255, 0.9);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.25);
  box-shadow: 0 4px 14px rgba(0, 122, 255, 0.3);
  
  /* HOVER STATE (mandatory) */
  &:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(0, 122, 255, 0.4);
  }
  
  /* ACTIVE STATE (mandatory) */
  &:active {
    transform: translateY(0);
  }
}
```

### 3️⃣ **FOR EVERY CARD/CONTAINER**

```css
.card {
  /* MUST HAVE */
  padding: 20px;
  border-radius: 16px;
  background: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  
  /* OPTIONAL GLASS */
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(40px);
  border: 1px solid rgba(255, 255, 255, 0.18);
}
```

### 4️⃣ **FOR EVERY INPUT**

```css
.input {
  /* MUST HAVE */
  padding: 12px 16px;
  border-radius: 10px;
  font-size: 17px;
  background: #F2F2F7;
  border: 1px solid transparent;
  transition: all 0.2s;
  
  /* FOCUS STATE (mandatory) */
  &:focus {
    outline: none;
    background: white;
    border-color: #007AFF;
    box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.1);
  }
}
```

---

## 🎨 Color Usage Rules

### Primary Actions → Blue
```css
background: #007AFF;
```

### Secondary Actions → Gray
```css
background: #8E8E93;
```

### Destructive Actions → Red
```css
background: #FF3B30;
```

### Success States → Green
```css
background: #34C759;
```

### Backgrounds → Grays
```css
/* Main background */
background: #FFFFFF;

/* Elevated/Card background */
background: #F2F2F7;

/* Input background */
background: #F2F2F7;
```

---

## 📐 Spacing Rules

### ALWAYS use these values:
- `4px` - Micro spacing (icons)
- `8px` - Tight spacing
- `12px` - Button padding vertical
- `16px` - Default spacing
- `20px` - Card padding
- `24px` - Section spacing
- `32px` - Large spacing
- `48px` - Extra large spacing

### NEVER use:
- Odd numbers (except established: 15px, 17px for typography)
- Random values
- Values not divisible by 4 (except typography)

---

## 🔄 Animation Rules

### ALWAYS animate with:
```css
/* Transform for movement */
transform: translateY(-2px);

/* Opacity for fading */
opacity: 0.8;

/* Scale for growth */
transform: scale(1.02);
```

### NEVER animate:
- Width/Height (use transform: scale)
- Top/Left/Right/Bottom (use transform: translate)
- Padding/Margin (causes reflow)

### Timing:
- `100ms` - Instant feedback
- `200ms` - Default transitions
- `300ms` - Deliberate animations
- `500ms` - Slow reveals

---

## 🚫 Common Mistakes to AVOID

### ❌ DON'T DO THIS:
```css
/* Sharp corners */
border-radius: 0;

/* Harsh shadows */
box-shadow: 0 10px 20px rgba(0,0,0,0.5);

/* No transitions */
/* (missing transition property) */

/* Thick borders */
border: 3px solid;

/* Pure black/white */
color: #000000;
background: #FFFFFF;
```

### ✅ DO THIS INSTEAD:
```css
/* Rounded corners */
border-radius: 12px;

/* Soft shadows */
box-shadow: 0 4px 14px rgba(0,0,0,0.08);

/* Smooth transitions */
transition: all 0.2s ease;

/* Subtle borders */
border: 1px solid rgba(255,255,255,0.18);

/* Soft blacks/whites */
color: #1C1C1E;
background: #FAFAFA;
```

---

## 🎯 Quick Decision Tree

### Need to create a component?

1. **Is it clickable?** → Add hover lift + shadow change
2. **Is it a container?** → Add soft shadow + rounded corners
3. **Does it overlay content?** → Add glass effect
4. **Is it a form element?** → Add focus ring
5. **Does it need emphasis?** → Add colored shadow
6. **Is it loading?** → Add pulse animation

---

## 💻 Copy-Paste Templates

### Glass Button
```css
.btn-glass {
  padding: 10px 20px;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.1);
}

.btn-glass:hover {
  transform: translateY(-2px);
  background: rgba(255, 255, 255, 0.3);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
}
```

### Glass Card
```css
.card-glass {
  padding: 20px;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(40px);
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}
```

### Modern Input
```css
.input-modern {
  width: 100%;
  padding: 12px 16px;
  background: #F2F2F7;
  border: 1px solid transparent;
  border-radius: 10px;
  font-size: 17px;
  transition: all 0.2s ease;
}

.input-modern:focus {
  outline: none;
  background: white;
  border-color: #007AFF;
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.1);
}
```

---

## 🔍 Final Checklist

Before delivering any UI component, verify:

- [ ] **Spacing**: Uses 4px grid system
- [ ] **Colors**: From Apple palette
- [ ] **Borders**: Rounded (8px, 10px, 12px, or 16px)
- [ ] **Shadows**: Soft and multi-layered
- [ ] **Typography**: -apple-system font
- [ ] **States**: Hover, active, focus defined
- [ ] **Animation**: Smooth transitions (200-300ms)
- [ ] **Glass**: Applied where appropriate
- [ ] **Accessibility**: Focus rings present
- [ ] **Performance**: Uses transforms not position

---

## 🚀 Implementation Order

1. **Structure** (HTML)
2. **Layout** (Flexbox/Grid)
3. **Spacing** (Padding/Margin)
4. **Colors** (Background/Text)
5. **Typography** (Font/Size/Weight)
6. **Borders** (Radius/Color)
7. **Shadows** (Elevation)
8. **Glass Effects** (Blur/Transparency)
9. **Transitions** (Hover/Focus)
10. **Polish** (Micro-animations)

---

**Remember**: Every pixel matters. If it doesn't look premium, it's not done.

