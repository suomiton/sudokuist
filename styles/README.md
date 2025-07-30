# CSS Architecture Documentation

## Overview

The Sudokuist application uses a modular CSS architecture with CSS nesting to improve maintainability, prevent style leakage, and provide better organization.

## File Structure

```
styles/
├── main.css              # Main entry point (imports all modules)
├── variables.css         # CSS custom properties and design tokens
├── base.css             # Base styles, resets, and utility classes
├── animations.css       # Keyframe animations and animation utilities
├── responsive.css       # Responsive design and media queries
└── components/
    ├── modal.css        # Modal component styles
    ├── difficulty-selection.css  # Difficulty selection UI
    ├── start-screen.css # Start screen layout
    ├── game-board.css   # Sudoku board and game container
    ├── timer.css        # Timer component
    └── share.css        # Share functionality styles
```

## Architecture Principles

### 1. **Modular Design**

- Each component has its own CSS file
- Components are self-contained with minimal external dependencies
- Easy to maintain and update individual features

### 2. **CSS Nesting**

- Uses modern CSS nesting for better organization
- Prevents style leakage through scoped selectors
- Easier to read and maintain nested component styles

### 3. **Design System Variables**

- Centralized design tokens in `variables.css`
- Consistent spacing using 8px grid system
- Semantic color naming with nature-inspired palette
- Reusable shadows, transitions, and typography scales

### 4. **BEM-like Naming with Nesting**

- Component-based naming convention
- CSS nesting reduces repetition and improves readability
- Clear hierarchy and relationships between elements

## Key Features

### Design Tokens (`variables.css`)

```css
:root {
	/* Typography */
	--font-family: "Share Tech Mono", monospace;
	--font-size-base: 16px;

	/* Spacing (8px grid) */
	--gutter-base: 8px;
	--gutter-sm: 4px;
	--gutter-md: 8px;
	--gutter-lg: 16px;
	--gutter-xl: 24px;
	--gutter-2xl: 32px;

	/* Semantic Colors */
	--color-primary: var(--color-hunter-green);
	--color-error: #d32f2f;
	--color-success: var(--color-fern-green);
	/* ... */
}
```

### Component Structure Example

```css
.modal {
	&-overlay {
		/* Overlay styles */

		&-visible {
			/* Visible state */
		}
	}

	&-container {
		/* Container styles */

		&-visible {
			/* Visible state */
		}
	}

	&-header {
		/* Header styles */
	}
}
```

### Responsive Design

- Mobile-first approach
- Flexible breakpoints for different screen sizes
- High-DPI display optimization
- Component-specific responsive behavior

## Benefits

1. **Maintainability**: Each component is isolated and easy to modify
2. **Scalability**: Easy to add new components without affecting existing styles
3. **Performance**: Modular loading allows for potential code splitting
4. **Developer Experience**: Clear organization and CSS nesting improve readability
5. **Design Consistency**: Centralized design tokens ensure consistent styling
6. **Reduced Specificity Issues**: Component-scoped styles prevent conflicts

## Usage

The main entry point is `styles/main.css`, which imports all necessary modules:

```html
<link rel="stylesheet" href="/styles/main.css" />
```

## Future Enhancements

- **CSS Layers**: Consider using `@layer` for better cascade control
- **Component Variants**: Extend the design system with more component variants
- **Dark Mode**: Leverage CSS custom properties for theme switching
- **CSS Modules**: Consider build-time optimization for production
- **Container Queries**: Use container queries for more responsive components

## Browser Support

- Modern browsers supporting CSS nesting (Chrome 112+, Firefox 117+, Safari 16.5+)
- Fallbacks provided where necessary
- Progressive enhancement approach for older browsers
