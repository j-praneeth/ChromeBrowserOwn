# Privacy Browser

## Overview

Privacy Browser is a desktop web browser application built with Tauri, combining a Rust backend with a web-based frontend using HTML, CSS, and JavaScript. The application focuses on user privacy by implementing ad and tracker blocking capabilities, providing a secure browsing experience with modern browser features like tabbed browsing, navigation controls, and privacy indicators.

## System Architecture

### Frontend Architecture
- **Technology Stack**: Vanilla HTML5, CSS3, and JavaScript (ES6+)
- **UI Framework**: Custom implementation with Font Awesome icons
- **Layout**: Single-page application with modular components
- **Styling**: Custom CSS with dark theme and modern UI patterns

### Backend Architecture
- **Framework**: Tauri (Rust-based desktop application framework)
- **Architecture Pattern**: Event-driven with frontend-backend communication via Tauri APIs
- **Security Model**: Restricted API access with specific allowlist permissions

### Desktop Integration
- **Window Management**: Custom window controls (minimize, maximize, close)
- **File System Access**: Limited scope to application data and resources
- **HTTP Requests**: Enabled for web content fetching
- **Notifications**: Full notification system support

## Key Components

### 1. Browser Core (`script.js`)
- **PrivacyBrowser Class**: Main application controller managing tabs, navigation, and privacy features
- **Tab Management**: Dynamic tab creation, switching, and closure
- **Navigation System**: Back/forward history, address bar, and page refresh
- **Privacy Protection**: Request blocking and tracking prevention

### 2. Privacy Engine (`privacy-lists.js`)
- **Tracking Domain Lists**: Comprehensive database of known tracking and advertising domains
- **Blocking Rules**: Static list-based approach for privacy protection
- **Categories**: Covers analytics, social media, advertising, and telemetry services

### 3. User Interface (`index.html`, `styles.css`)
- **Window Controls**: Custom titlebar with minimize/maximize/close functionality
- **Tab Bar**: Dynamic tab management interface
- **Navigation Bar**: Address bar with security indicators and privacy counters
- **Dark Theme**: Modern dark UI design with consistent styling

### 4. Tauri Configuration
- **Security Permissions**: Minimal required permissions for browser functionality
- **File System**: Scoped access to application data directories
- **HTTP Access**: Unrestricted for web content loading
- **Shell Integration**: Limited to opening external applications

## Data Flow

### 1. Application Startup
1. Tauri runtime initializes with restricted permissions
2. Frontend loads and creates PrivacyBrowser instance
3. Initial tab creation and start page display
4. Bookmark and history data loading from local storage

### 2. Web Navigation
1. User input in address bar triggers URL validation
2. Privacy lists check against target domain
3. HTTP requests filtered through blocking rules
4. Page content loaded in active tab
5. Privacy counter updates with blocked requests

### 3. Tab Management
1. New tab creation generates unique tab identifier
2. Tab state stored in browser's tabs Map
3. Active tab switching updates UI and content display
4. Tab closure removes from memory and UI

## External Dependencies

### Frontend Dependencies
- **Font Awesome 6.0.0**: Icon library loaded via CDN
- **No additional JavaScript frameworks**: Vanilla JS implementation

### Backend Dependencies
- **Tauri Framework**: Desktop application runtime
- **Rust Ecosystem**: Underlying system integration

### Privacy Lists
- **Static Domain Lists**: Self-maintained tracking domain database
- **No External APIs**: Privacy protection works offline

## Deployment Strategy

### Development Environment
- **Build System**: Tauri's integrated build pipeline
- **Development Server**: Direct file serving from `src` directory
- **Hot Reload**: Supported through Tauri's development mode

### Production Build
- **Target Platforms**: Cross-platform support (Windows, macOS, Linux)
- **Bundle Strategy**: Single executable with embedded web assets
- **Icon Resources**: Multiple resolution support for different platforms
- **Distribution**: Standalone application bundles

### Configuration Management
- **Dual Configuration**: Separate configs for root and src-tauri directories
- **Version Management**: Centralized version control in package configuration
- **Permission Model**: Explicit allowlist-based security configuration

## Changelog
- July 02, 2025. Initial setup
- July 02, 2025. Fixed Tauri compilation issues and system dependencies
- July 02, 2025. Successfully building native cross-platform privacy browser
- July 02, 2025. Fixed Tauri configuration features and created application icons
- July 02, 2025. Native browser compilation in final stage with all dependencies resolved
- July 02, 2025. **Web demo fully functional** - Fixed JavaScript initialization, tab creation, navigation, and iframe loading with proper fallback for blocked sites

## User Preferences

Preferred communication style: Simple, everyday language.
User wants native application (not web-based demo) with Chrome-level performance.