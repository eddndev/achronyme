# Contributing to Achronyme

Thank you for your interest in contributing to **Achronyme**! We're building the next-generation open-source engineering toolbox, and we're excited to have you join us.

## 🌟 Vision

Achronyme aims to be the definitive, modern, web-based toolbox for engineering students and professionals. We're creating free, fast, and accessible mathematical tools to replace expensive proprietary software.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Submitting Changes](#submitting-changes)
- [Roadmap](#roadmap)

## 📜 Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow:

- **Be respectful** and inclusive
- **Be collaborative** and constructive
- **Focus on what's best** for the community
- **Show empathy** towards other community members

Unacceptable behavior will not be tolerated.

## 🤝 How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the issue
- **Expected vs actual behavior**
- **Screenshots** (if applicable)
- **Browser and OS** information

### Suggesting Features

We love new ideas! When suggesting features:

- **Check existing suggestions** first
- **Explain the use case** - why is this useful for engineers?
- **Provide examples** of how it would work
- **Consider implementation** - is it feasible?

### Adding New Tools

Want to add a new mathematical/engineering tool? Great! Please:

1. **Open an issue first** to discuss the tool
2. **Provide mathematical background** and use cases
3. **Include test cases** with expected results
4. **Add documentation** explaining the math and usage

### Improving Documentation

Documentation is crucial! You can help by:

- Fixing typos or unclear explanations
- Adding examples and use cases
- Translating to other languages (español, português, français, etc.)
- Creating video tutorials

### Code Contributions

We welcome code contributions! Areas where you can help:

- **Frontend improvements** (UI/UX, animations, responsiveness)
- **Mathematical algorithms** (optimization, accuracy improvements)
- **Performance** (WASM implementations, code optimization)
- **Testing** (unit tests, integration tests)
- **Accessibility** (ARIA, keyboard navigation, screen readers)

## 🛠️ Development Setup

### Prerequisites

- **PHP** >= 8.2
- **Composer** 2.x
- **Node.js** >= 18.x
- **npm** >= 9.x
- **MySQL** >= 8.0 or **MariaDB** >= 10.3

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/eddndev/achronyme.git
   cd achronyme
   ```

2. **Install PHP dependencies**
   ```bash
   composer install
   ```

3. **Install Node dependencies**
   ```bash
   npm install
   ```

4. **Setup environment**
   ```bash
   cp .env.example .env
   php artisan key:generate
   ```

5. **Configure database**

   Edit `.env` with your database credentials:
   ```env
   DB_DATABASE=achronyme
   DB_USERNAME=your_username
   DB_PASSWORD=your_password
   ```

6. **Run migrations**
   ```bash
   php artisan migrate
   ```

7. **Build assets**
   ```bash
   npm run dev
   ```

8. **Start development server**
   ```bash
   php artisan serve
   ```

   Visit: `http://localhost:8000`

### Development Commands

```bash
# Watch for asset changes
npm run dev

# Build for production
npm run build

# Run PHP linter
./vendor/bin/pint

# Run tests
php artisan test
```

## 📁 Project Structure

```
achronyme/
├── app/
│   ├── Http/Controllers/      # Controllers
│   ├── Models/                # Eloquent models
│   └── View/Components/       # Blade components
├── resources/
│   ├── views/                 # Blade templates
│   │   ├── auth/             # Authentication views
│   │   ├── components/       # Reusable components
│   │   └── tools/            # Tool-specific views
│   ├── js/                   # JavaScript modules
│   │   ├── fs/              # Fourier Series
│   │   ├── ft/              # Fourier Transform
│   │   └── convolution/     # Convolution
│   └── css/                  # Stylesheets
├── routes/
│   └── web.php              # Web routes
├── database/
│   └── migrations/          # Database migrations
└── tests/                   # Test files
```

## 🎨 Coding Standards

### PHP (Laravel)

- Follow **PSR-12** coding standard
- Use **Laravel best practices**
- Run `./vendor/bin/pint` before committing
- Write **docblocks** for all public methods
- Use **type hints** and **return types**

Example:
```php
/**
 * Calculate Fourier series coefficients
 *
 * @param array $data Input signal data
 * @param int $harmonics Number of harmonics
 * @return array Fourier coefficients
 */
public function calculateCoefficients(array $data, int $harmonics): array
{
    // Implementation
}
```

### JavaScript

- Use **ES6+ syntax**
- Follow **Airbnb style guide**
- Add **JSDoc comments** for functions
- Keep functions **pure** when possible
- Use **async/await** over promises

Example:
```javascript
/**
 * Compute Fast Fourier Transform
 * @param {number[]} signal - Input signal array
 * @returns {Complex[]} FFT result
 */
async function computeFFT(signal) {
    // Implementation
}
```

### CSS/Tailwind

- Use **Tailwind utility classes** first
- Create **custom components** only when necessary
- Follow **mobile-first** approach
- Keep **dark mode** in mind
- Use **semantic color names**

### Blade Templates

- Keep logic **minimal** in views
- Use **components** for reusability
- Follow **accessibility best practices**
- Add **ARIA attributes** where needed

## 🔬 Mathematical Accuracy

When implementing mathematical algorithms:

1. **Reference authoritative sources**
   - Link to papers, textbooks, or standards
   - Cite algorithms and formulas

2. **Include test cases**
   - Known inputs with expected outputs
   - Edge cases and boundary conditions
   - Compare with established tools (MATLAB, NumPy, etc.)

3. **Document limitations**
   - Numerical precision constraints
   - Performance characteristics
   - Valid input ranges

4. **Add examples**
   - Real-world use cases
   - Step-by-step explanations
   - Interactive demos

## 📤 Submitting Changes

### Pull Request Process

1. **Fork the repository**

2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-tool
   ```

3. **Make your changes**
   - Write clean, documented code
   - Add tests if applicable
   - Update documentation

4. **Test your changes**
   ```bash
   php artisan test
   npm run build
   ```

5. **Commit with clear messages**
   ```bash
   git commit -m "feat: add matrix determinant calculator"
   ```

   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation only
   - `style:` Code style (formatting, missing semicolons, etc.)
   - `refactor:` Code refactoring
   - `perf:` Performance improvement
   - `test:` Adding tests
   - `chore:` Maintenance tasks

6. **Push to your fork**
   ```bash
   git push origin feature/amazing-tool
   ```

7. **Open a Pull Request**
   - Use a clear, descriptive title
   - Reference related issues
   - Describe what and why (not how)
   - Add screenshots for UI changes
   - Check all CI/CD checks pass

### PR Review Process

- Maintainers will review within **1 week**
- Address feedback professionally
- Be patient - quality takes time
- Once approved, we'll merge!

## 🗺️ Roadmap

### Current Focus (Q1 2025)
- ✅ Fourier Series & Transform
- ✅ Convolution
- 🔄 Quantitative Methods for Decision Making
- 🔄 Linear Programming (Simplex method)

### Near Future (Q2-Q3 2025)
- Numerical Analysis tools
- Differential Equations solvers
- Statistics & Probability tools
- Matrix operations & Linear Algebra

### Long Term (Q4 2025+)
- WebAssembly (WASM) migration for performance
- Mobile app (PWA)
- API for external integrations
- Free courses & tutorials
- Multi-language support

## 💬 Community

- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: General questions and ideas
- **Email**: [contacto@eddndev.com](mailto:contacto@eddndev.com)
- **Facebook**: [@eddndev.studios](https://facebook.com/eddndev.studios)
- **Instagram**: [@_eddndev](https://instagram.com/_eddndev)
- **TikTok**: [@eddndev](https://tiktok.com/@eddndev)
- **YouTube**: [@eddndev](https://youtube.com/@eddndev)
- **GitHub**: [@eddndev](https://github.com/eddndev)

## 📄 License

By contributing, you agree that your contributions will be licensed under the same license as the project (see [LICENSE](LICENSE) file).

## 🙏 Thank You!

Every contribution, no matter how small, makes Achronyme better for engineers worldwide. Thank you for being part of this journey!

---

**Built with ❤️ by the Achronyme community**