# Manifest - Achronyme

**Versión:** 1.0
**Fecha:** 2025-10-05
**Mantenido por:** @eddndev

---

## 1. Visión del Proyecto

**Achronyme** es una plataforma web open-source que democratiza el acceso a herramientas matemáticas y de ingeniería de alto rendimiento. Nuestro objetivo es reemplazar software propietario costoso con una alternativa moderna, rápida y accesible basada en la web.

### Propuesta de Valor

- **100% Gratuito y Open Source**: Sin licencias ni muros de pago
- **Alto Rendimiento**: Optimización con WebAssembly y C++ para cálculos críticos
- **Accesibilidad Universal**: Funciona en cualquier dispositivo con navegador
- **Educativo**: Incluye tutoriales y documentación integrada
- **Moderno**: Interfaz intuitiva y hermosa

---

## 2. Objetivos del Proyecto

### Objetivos a Corto Plazo (Q1-Q2 2025)
1. Consolidar herramientas de Procesamiento Digital de Señales (DSP):
   - Series de Fourier
   - Transformada de Fourier
   - Convolución
2. Implementar sistema de autenticación y perfiles de usuario
3. Mejorar diseño responsive y accesibilidad

### Objetivos a Medio Plazo (Q3-Q4 2025)
1. Migrar operaciones matemáticas críticas a WebAssembly (C++)
2. Expandir herramientas:
   - Programación Lineal
   - Métodos Cuantitativos
   - Operaciones Matriciales
   - Estadística y Probabilidad
3. Implementar sistema de cursos gratuitos
4. Soporte multi-idioma (ES, EN, PT, FR)

### Objetivos a Largo Plazo (2026+)
1. API pública para integraciones
2. Sistema de plugins para herramientas personalizadas
3. Aplicaciones nativas móviles y de escritorio
4. Herramientas contribuidas por la comunidad

---

## 3. Alcance del Proyecto

### Dentro del Alcance

**Funcionalidades Core:**
- Herramientas matemáticas de ingeniería (DSP, álgebra lineal, cálculo numérico, etc.)
- Sistema de autenticación y gestión de usuarios
- Visualización interactiva de resultados (gráficas, animaciones)
- Renderizado de fórmulas matemáticas (LaTeX)
- Sistema de tutoriales educativos
- Multi-idioma
- API REST para integraciones
- Optimización con WebAssembly

**Stack Tecnológico:**
- Backend: Laravel 12, PHP 8.2+, MySQL/MariaDB
- Frontend: Vite, Tailwind CSS, Alpine.js, GSAP, Chart.js, Math.js, MathJax
- Performance: WebAssembly, C/C++, Emscripten

### Fuera del Alcance (por ahora)

- Cálculos simbólicos avanzados (como Mathematica/Maple)
- Simulaciones 3D complejas
- Computación distribuida/cloud computing
- Machine Learning integrado
- Blockchain/Web3

---

## 4. Stakeholders

### Core Team
- **Eduardo Alonso Sánchez** (@eddndev) - Founder & Lead Developer

### Contributors Originales (Proyecto Universitario DSP)
1. Alonso Sánchez Eduardo
2. Bonilla Ramírez Josué Eleazar
3. Jiménez Meza Ana Harumi
4. Quiroz Mora Abel Mauricio
5. Vilchis Paniagua Johan Emiliano

### Community
- Open-source contributors
- Engineering students and professionals (usuarios finales)
- Educational institutions

---

## 5. Riesgos y Mitigaciones

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Bajo rendimiento en cálculos complejos | Media | Alto | Migración a WebAssembly con C++ |
| Falta de contribuidores | Media | Medio | Documentación clara, good first issues |
| Complejidad de la migración WASM | Alta | Alto | Implementación incremental, testing riguroso |
| Escalabilidad del backend | Baja | Medio | Arquitectura modular, caché, optimización DB |
| Compatibilidad cross-browser | Media | Medio | Testing en múltiples navegadores, polyfills |

---

## 6. Métricas de Éxito

### KPIs Técnicos
- **Performance**: Tiempo de cálculo <100ms para operaciones estándar
- **Uptime**: >99.5% disponibilidad
- **Test Coverage**: >80% cobertura de tests
- **Lighthouse Score**: >90 en todas las métricas

### KPIs de Usuario
- Usuarios activos mensuales (MAU)
- Tasa de retención (30 días)
- NPS (Net Promoter Score) >50
- Issues resueltas vs. reportadas

### KPIs de Comunidad
- Contributors activos
- Pull Requests mergeados/mes
- GitHub Stars
- Documentación actualizada (días sin cambios <30)

---

## 7. Restricciones y Asunciones

### Restricciones
- Presupuesto limitado (proyecto open-source)
- Equipo pequeño (inicialmente 1 desarrollador)
- Debe funcionar en navegadores modernos (últimas 2 versiones)
- Cumplir con WCAG 2.1 Level AA para accesibilidad

### Asunciones
- Los usuarios tienen acceso a internet estable
- Los usuarios usan navegadores con soporte WebAssembly
- La comunidad contribuirá activamente al proyecto
- Las herramientas matemáticas son la propuesta de valor principal

---

## 8. Arquitectura de Alto Nivel

```
┌─────────────────────────────────────────────────┐
│                   Frontend                      │
│  ┌──────────┬──────────┬─────────────────────┐ │
│  │  Vite    │ Tailwind │ Alpine.js / GSAP    │ │
│  └──────────┴──────────┴─────────────────────┘ │
│  ┌──────────┬──────────┬─────────────────────┐ │
│  │ Math.js  │ Chart.js │     MathJax         │ │
│  └──────────┴──────────┴─────────────────────┘ │
└─────────────────┬───────────────────────────────┘
                  │ HTTP/REST
┌─────────────────▼───────────────────────────────┐
│                 Backend                         │
│  ┌──────────────────────────────────────────┐  │
│  │          Laravel 12 (PHP 8.2+)           │  │
│  │  ┌────────────┬─────────────────────────┐│  │
│  │  │ Controllers│   Services / Business   ││  │
│  │  │            │        Logic            ││  │
│  │  └────────────┴─────────────────────────┘│  │
│  └──────────────────┬───────────────────────┘  │
│                     │                           │
│  ┌──────────────────▼───────────────────────┐  │
│  │        MySQL / MariaDB Database          │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│         Future: WASM Layer (C/C++)              │
│  ┌──────────────────────────────────────────┐  │
│  │  High-Performance Mathematical Core      │  │
│  │  (Fourier, Linear Algebra, Numerical)    │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

---

## 9. Decisiones Técnicas Clave

### ¿Por qué Laravel?
- Ecosystem maduro y robusto
- Eloquent ORM para gestión de BD
- Sistema de autenticación integrado
- Excelente documentación
- Compatible con despliegue en múltiples plataformas

### ¿Por qué WebAssembly?
- Rendimiento near-native para cálculos pesados
- Portabilidad cross-platform
- Reutilización de código C++ existente
- Sin necesidad de backend para cálculos

### ¿Por qué Math.js inicialmente?
- Rápido prototipado
- API completa y fácil de usar
- Migración incremental a WASM

---

## 10. Roadmap de Alto Nivel

**Q1 2025 (Actual)**
- ✅ Implementación DSP (Fourier, Convolución)
- ⏳ Sistema de autenticación
- ⏳ Responsive design

**Q2-Q3 2025**
- Migración a WebAssembly (C++)
- Nuevas herramientas (Linear Programming, Quantitative Methods)
- Multi-idioma

**Q4 2025**
- Benchmarks de rendimiento
- PWA mobile-first
- API pública

**2026+**
- Sistema de plugins
- Apps nativas (mobile/desktop)
- Contribuciones de comunidad

---

**Documento vivo - Actualizado con cada decisión arquitectónica mayor**