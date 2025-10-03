<div align="center">
  <img src="./public/images/logo.png" alt="Achronyme Logo" width="200">
  <br/>
  <h1>
    <b>Achronyme</b>
  </h1>
  <p>
    <b>The Open Engineering Toolbox</b>
  </p>
  <p>
    Modern, fast, and free mathematical tools for engineers and students worldwide.
  </p>
</div>

<div align="center">
  <img src="https://img.shields.io/badge/PHP-%3E%3D8.2-8A2BE2?style=for-the-badge&logo=php&logoColor=white" alt="PHP version">
  <img src="https://img.shields.io/badge/Laravel-12.x-FF2D20?style=for-the-badge&logo=laravel&logoColor=white" alt="Laravel">
  <img src="https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/Tailwind_CSS-9370DB?style=for-the-badge&logo=tailwind-css&logoColor=white" alt="Tailwind CSS">
  <img src="https://img.shields.io/badge/License-MIT-9370DB?style=for-the-badge" alt="License MIT">
</div>

<div align="center">
  <a href="https://achrony.me">🌐 Live Demo</a> •
  <a href="#-features">Features</a> •
  <a href="#-getting-started">Getting Started</a> •
  <a href="CONTRIBUTING.md">Contributing</a> •
  <a href="#-roadmap">Roadmap</a>
</div>

---

## 🎯 Vision

**Achronyme** aims to replace expensive proprietary engineering software with a modern, web-based, open-source alternative. We're building a comprehensive toolbox that empowers engineering students and professionals with fast, accurate, and accessible mathematical tools.

### Why Achronyme?

- ✅ **100% Free & Open Source** - No licenses, no paywalls
- ⚡ **Blazing Fast** - Optimized algorithms, future WASM support
- 🌐 **Web-Based** - Works on any device with a browser
- 📚 **Educational** - Built-in tutorials and documentation
- 🎨 **Modern UI** - Beautiful, intuitive interface
- ♿ **Accessible** - WCAG compliant, keyboard navigation
- 🌍 **International** - Multi-language support (coming soon)

---

## ✨ Features

### Current Tools

#### 📊 **Digital Signal Processing**
- **Fourier Series** - Decompose periodic functions into sines and cosines
- **Fourier Transform** - Analyze signals in frequency domain
- **Convolution** - Visualize signal convolution in real-time

### Coming Soon

- 📐 **Linear Programming** - Simplex method solver
- 📈 **Quantitative Methods** - Decision-making tools
- 🧮 **Matrix Operations** - Linear algebra toolkit
- 📉 **Numerical Analysis** - Root finding, integration, differentiation
- 📊 **Statistics & Probability** - Comprehensive statistical tools
- 🔢 **Differential Equations** - ODE and PDE solvers

---

## 🚀 Getting Started

### Prerequisites

Make sure you have the following installed:

- **PHP** >= 8.2
- **Composer** >= 2.x
- **Node.js** >= 18.x
- **npm** >= 9.x
- **MySQL** >= 8.0 or **MariaDB** >= 10.3

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/eddndev/achronyme.git
   cd achronyme
   ```

2. **Install dependencies**
   ```bash
   composer install
   npm install
   ```

3. **Setup environment**
   ```bash
   cp .env.example .env
   php artisan key:generate
   ```

4. **Configure database**

   Edit `.env` with your database credentials:
   ```env
   DB_DATABASE=achronyme
   DB_USERNAME=your_username
   DB_PASSWORD=your_password
   ```

5. **Run migrations**
   ```bash
   php artisan migrate
   ```

6. **Build assets**
   ```bash
   npm run dev
   ```

7. **Start development server**
   ```bash
   php artisan serve
   ```

8. **Visit** `http://localhost:8000`

For detailed setup instructions, see [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 🏗️ Tech Stack

### Backend
- **Laravel 12** - Robust PHP framework
- **PHP 8.2+** - Modern PHP with types and attributes
- **MySQL/MariaDB** - Reliable database

### Frontend
- **Vite** - Lightning-fast build tool
- **Tailwind CSS** - Utility-first CSS framework
- **Alpine.js** - Lightweight JavaScript framework
- **GSAP** - Professional-grade animation
- **Chart.js** - Beautiful, responsive charts
- **Math.js** - Comprehensive math library
- **MathJax** - LaTeX formula rendering

### Future
- **WebAssembly (WASM)** - High-performance computing
- **C/C++** - Core mathematical algorithms
- **Rust** - System-level optimizations

---

## 📖 Documentation

### User Guide
- [Fourier Series Guide](docs/fourier-series.md) _(coming soon)_
- [Fourier Transform Guide](docs/fourier-transform.md) _(coming soon)_
- [Convolution Guide](docs/convolution.md) _(coming soon)_

### Developer Docs
- [Contributing Guide](CONTRIBUTING.md)
- [API Reference](docs/api.md) _(coming soon)_
- [Architecture Overview](docs/architecture.md) _(coming soon)_

---

## 🗺️ Roadmap

### Phase 1: Foundation (Current - Q1 2025)
- [x] Fourier Series implementation
- [x] Fourier Transform implementation
- [x] Convolution visualization
- [ ] User authentication & profiles
- [ ] Responsive design improvements

### Phase 2: Expansion (Q2-Q3 2025)
- [ ] Linear Programming tools
- [ ] Quantitative Methods suite
- [ ] Matrix operations & Linear Algebra
- [ ] Statistics & Probability tools
- [ ] Free courses & tutorials
- [ ] Multi-language support (ES, EN, PT, FR)

### Phase 3: Performance (Q4 2025)
- [ ] WebAssembly migration for critical operations
- [ ] C/C++ core mathematical library
- [ ] Performance benchmarks
- [ ] Mobile-first PWA

### Phase 4: Community (2026+)
- [ ] Public API for integrations
- [ ] Plugin system for custom tools
- [ ] Community-contributed tools
- [ ] Mobile native apps
- [ ] Desktop apps (Electron/Tauri)

---

## 🤝 Contributing

We welcome contributions from everyone! Whether you're fixing bugs, adding features, improving documentation, or suggesting ideas - your help makes Achronyme better.

Please read our [Contributing Guide](CONTRIBUTING.md) to get started.

### Ways to Contribute

- 🐛 **Report bugs** - Help us improve quality
- 💡 **Suggest features** - Share your ideas
- 📝 **Improve docs** - Make things clearer
- 🌍 **Translate** - Make Achronyme accessible to more people
- 💻 **Write code** - Add tools and features
- 🎨 **Design** - Improve UI/UX

---

## 👥 Team

### Core Team

**Eduardo Alonso Sánchez**
- 🔗 [GitHub](https://github.com/eddndev)
- 📧 [contacto@eddndev.com](mailto:contacto@eddndev.com)
- 📱 [Instagram](https://instagram.com/_eddndev) • [Facebook](https://facebook.com/eddndev.studios) • [TikTok](https://tiktok.com/@eddndev) • [YouTube](https://youtube.com/@eddndev)
- Role: Founder & Lead Developer

### Original Contributors (Digital Signal Processing Project)

This project originated as a collaborative university project for Digital Signal Processing:

| # | Name |
|:-:|:-----|
| 1 | Alonso Sánchez Eduardo |
| 2 | Bonilla Ramírez Josué Eleazar |
| 3 | Jiménez Meza Ana Harumi |
| 4 | Quiroz Mora Abel Mauricio |
| 5 | Vilchis Paniagua Johan Emiliano |

### Community Contributors

See [CONTRIBUTORS.md](CONTRIBUTORS.md) for our amazing community members!

---

## 📊 Project Stats

![GitHub stars](https://img.shields.io/github/stars/eddndev/achronyme?style=social)
![GitHub forks](https://img.shields.io/github/forks/eddndev/achronyme?style=social)
![GitHub issues](https://img.shields.io/github/issues/eddndev/achronyme)
![GitHub pull requests](https://img.shields.io/github/issues-pr/eddndev/achronyme)

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

### What this means:
- ✅ Use commercially
- ✅ Modify freely
- ✅ Distribute
- ✅ Private use
- ❌ Liability
- ❌ Warranty

---

## 🙏 Acknowledgments

- **Math.js** - Comprehensive mathematics library
- **Chart.js** - Beautiful charting library
- **MathJax** - Mathematical formula rendering
- **GSAP** - Professional animation platform
- **Laravel** - The PHP framework for web artisans
- **Tailwind CSS** - Utility-first CSS framework

Special thanks to all our [contributors](CONTRIBUTORS.md)!

---

## 💬 Community & Support

- 🌐 **Website**: [achrony.me](https://achrony.me)
- 🐛 **Issues**: [GitHub Issues](https://github.com/eddndev/achronyme/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/eddndev/achronyme/discussions)
- 📧 **Email**: [contacto@eddndev.com](mailto:contacto@eddndev.com)

### Follow Us

- 📷 **Instagram**: [@_eddndev](https://instagram.com/_eddndev)
- 📘 **Facebook**: [@eddndev.studios](https://facebook.com/eddndev.studios)
- 🎵 **TikTok**: [@eddndev](https://tiktok.com/@eddndev)
- 🎥 **YouTube**: [@eddndev](https://youtube.com/@eddndev)
- 💻 **GitHub**: [@eddndev](https://github.com/eddndev)

---

## ⭐ Show Your Support

If you find Achronyme useful, please consider:

- ⭐ **Star this repository**
- 🐦 **Share on social media**
- 📝 **Write a blog post**
- 💰 **Sponsor the project** _(coming soon)_

---

<div align="center">
  <p>
    <b>Built with ❤️ by engineers, for engineers</b>
  </p>
  <p>
    <sub>Making engineering mathematics accessible to everyone, everywhere.</sub>
  </p>
</div>