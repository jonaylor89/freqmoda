# FreqModa Development Commands

# List available recipes
default:
    @just --list

# Install dependencies
install:
    npm install

# Run dev server (requires streaming-engine running on port 8080)
dev:
    npm run dev

# Build for production
build:
    npm run build

# Preview production build
preview:
    npm run preview

# Type check
check:
    npm run check
