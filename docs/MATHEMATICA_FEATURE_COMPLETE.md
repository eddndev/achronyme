# 📊 Mathematica Complete Feature Analysis

**Objetivo:** Documentar TODAS las capacidades de Wolfram Mathematica para diseñar Achronyme como competidor open-source.

**Versión de referencia:** Mathematica 14.3 (Agosto 2025)
**Funciones totales:** 6,602 funciones built-in + 3,000+ en Function Repository

---

## 🎯 Estadísticas Clave

| Métrica | Mathematica | Achronyme Target |
|---------|-------------|------------------|
| **Funciones totales** | ~6,600 | ~2,000-3,000 (core funcionalidad) |
| **Licencia** | Comercial ($495-$2,495) | Open Source (MIT/Apache 2.0) |
| **Runtime** | Kernel nativo | WASM (navegador + Node.js) |
| **Velocidad** | Baseline | 10-20x más rápido (WASM optimizado) |
| **Tamaño** | ~7GB instalación | <5MB WASM compressed |
| **Deployment** | Desktop only | Web + Desktop + Mobile |

---

## 📚 CATEGORÍAS COMPLETAS DE FUNCIONALIDAD

### 1. ⚡ CÁLCULO SIMBÓLICO Y NUMÉRICO

#### 1.1 Álgebra Elemental y Polinomios
- **Operaciones básicas:** `+`, `-`, `*`, `/`, `^`
- **Factorización:** `Factor[expr]`, `FactorTerms`, `FactorSquareFree`
- **Expansión:** `Expand[expr]`, `ExpandAll`, `ExpandDenominator`
- **Simplificación:** `Simplify[expr]`, `FullSimplify`, `Refine`
- **Manipulación de expresiones:** `Collect`, `Together`, `Apart`, `Cancel`
- **Raíces de polinomios:** `Solve`, `NSolve`, `Root`, `Roots`
- **División polinomial:** `PolynomialQuotient`, `PolynomialRemainder`, `PolynomialGCD`
- **Interpolación:** `InterpolatingPolynomial`, `LagrangeInterpolation`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO - fundamento del CAS)

#### 1.2 Cálculo Diferencial
- **Límites:** `Limit[expr, x->a]`, `Limit[expr, x->Infinity]`
  - Límites unilaterales: `Direction -> "FromAbove"`, `"FromBelow"`
  - Límites multivariables
- **Derivadas:** `D[expr, x]`, `D[expr, {x, n}]` (n-ésima derivada)
  - Derivadas parciales: `D[f[x,y], x, y]`
  - Derivadas direccionales: `Grad`, `Div`, `Curl`, `Laplacian`
  - Derivadas totales: `Dt[expr]`
- **Diferenciales:** `Differential[f[x]]`
- **Notación de Leibniz:** Soporte completo para `dy/dx`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO)

#### 1.3 Cálculo Integral
- **Integrales indefinidas:** `Integrate[f[x], x]`
- **Integrales definidas:** `Integrate[f[x], {x, a, b}]`
- **Integrales múltiples:** `Integrate[f[x,y], {x, a, b}, {y, c, d}]`
- **Integrales impropias:** Soporte automático para límites infinitos
- **Integrales de línea:** `LineIntegrate[field, path]`
- **Integrales de superficie:** `SurfaceIntegrate[field, surface]`
- **Integrales de volumen:** Integrales triples
- **Métodos numéricos:**
  - `NIntegrate` con métodos: Simpson, Trapezoidal, Gaussian Quadrature, Monte Carlo
  - Integración adaptativa automática
- **Transformaciones:** `IntegralTransform`, `FourierTransform`, `LaplaceTransform`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO)

#### 1.4 Series y Sumas
- **Series de Taylor:** `Series[f[x], {x, a, n}]`
- **Series de Laurent:** Expansión automática
- **Series de Fourier:** `FourierSeries[f[t], t, n]`
- **Sumas finitas:** `Sum[expr, {i, 1, n}]`
- **Sumas infinitas:** `Sum[expr, {i, 1, Infinity}]`
- **Productos:** `Product[expr, {i, 1, n}]`
- **Análisis asintótico:** `AsymptoticDSolveValue`, `AsymptoticIntegrate`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

---

### 2. 📐 ECUACIONES DIFERENCIALES

#### 2.1 Ecuaciones Diferenciales Ordinarias (ODE)
- **Simbólicas:** `DSolve[ode, y[x], x]`
  - Primer orden: lineales, separables, exactas, Bernoulli
  - Segundo orden: homogéneas, no-homogéneas, Euler-Cauchy
  - Orden n: Sistemas de ODEs
- **Numéricas:** `NDSolve[ode, y, {x, 0, 10}]`
  - Métodos: Runge-Kutta, Adams, BDF (Backward Differentiation)
  - Eventos de detección: `WhenEvent`
  - Stiff equations: Detección automática
- **Condiciones iniciales/frontera:** Soporte completo

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO para ingeniería)

#### 2.2 Ecuaciones Diferenciales Parciales (PDE)
- **Simbólicas:** `DSolve` para PDEs específicas (ondas, calor, Laplace)
- **Numéricas:** `NDSolve` con métodos de elementos finitos (FEM)
  - **Métodos:** Finite Element Method, Finite Difference Method
  - **Tipos:** Elípticas, parabólicas, hiperbólicas
  - **Geometrías:** 1D, 2D, 3D, dominios arbitrarios
  - **Condiciones:** Dirichlet, Neumann, Robin, mixtas

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta - diferenciador clave)

#### 2.3 Ecuaciones Diferenciales Especiales
- **Ecuaciones de retardo (DDE):** Delay Differential Equations
- **Ecuaciones diferenciales estocásticas (SDE):** Stochastic DEs
- **Ecuaciones diferenciales algebraicas (DAE):** Differential-Algebraic Equations
- **Ecuaciones fraccionarias:** Fractional order derivatives
- **Ecuaciones de diferencias:** Discrete equations

**Achronyme Priority:** ⭐⭐⭐ (Media - avanzado)

---

### 3. 🔢 ÁLGEBRA LINEAL

#### 3.1 Operaciones Matriciales Básicas
- **Construcción:** `{{1,2},{3,4}}`, `Array`, `Table`, `IdentityMatrix`, `DiagonalMatrix`
- **Aritmética:** `+`, `-`, `*`, `.` (producto matricial), `^` (potencia)
- **Transpuesta:** `Transpose[A]`, `ConjugateTranspose`
- **Inversa:** `Inverse[A]`, `PseudoInverse`
- **Determinante:** `Det[A]`
- **Traza:** `Tr[A]`
- **Normas:** `Norm[v]`, `Norm[A, p]` (p-normas)
- **Rango:** `MatrixRank[A]`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO)

#### 3.2 Sistemas de Ecuaciones Lineales
- **Solución exacta:** `LinearSolve[A, b]`
- **Solución numérica:** Métodos iterativos (Conjugate Gradient, GMRES)
- **Least squares:** `LeastSquares[A, b]`
- **Espacios nulos:** `NullSpace[A]`
- **Row reduction:** `RowReduce[A]`, `ReducedRowEchelonForm`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO - Simplex, optimización)

#### 3.3 Descomposiciones Matriciales
- **LU:** `LUDecomposition[A]`
- **QR:** `QRDecomposition[A]`
- **SVD:** `SingularValueDecomposition[A]`
- **Cholesky:** `CholeskyDecomposition[A]` (matrices simétricas positivas)
- **Schur:** `SchurDecomposition[A]`
- **Eigendecomposición:** `Eigensystem[A]`, `Eigenvalues`, `Eigenvectors`
- **Jordan:** `JordanDecomposition[A]`
- **Hessenberg:** `HessenbergDecomposition[A]`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

#### 3.4 Matrices Especiales
- **Sparse matrices:** `SparseArray` con operaciones optimizadas
- **Structured matrices:** Toeplitz, Hankel, Circulant
- **Matrices simbólicas:** Álgebra matricial simbólica completa
- **Bandas:** `UpperTriangularize`, `LowerTriangularize`, `BandMatrix`

**Achronyme Priority:** ⭐⭐⭐ (Media)

#### 3.5 Propiedades Matriciales
- **Tests:** `MatrixQ`, `SymmetricMatrixQ`, `DiagonalMatrixQ`, `PositiveDefiniteMatrixQ`
- **Características:** `MatrixExp`, `MatrixPower`, `MatrixFunction`
- **Normas:** Frobenius, operador, nuclear

**Achronyme Priority:** ⭐⭐⭐ (Media)

---

### 4. 🎲 OPTIMIZACIÓN

#### 4.1 Programación Lineal
- **Minimización/Maximización:** `LinearOptimization[c, A, b]`
- **Simplex method:** Implementación optimizada
- **Dual simplex:** Para problemas duales
- **Variables enteras:** Mixed Integer Linear Programming (MILP)
- **Problemas de transporte:** Specialized solvers

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO - tu caso de uso principal)

#### 4.2 Optimización No Lineal
- **Sin restricciones:** `FindMinimum[f[x], {x, x0}]`, `NMinimize`
- **Con restricciones:** `FindMinimum[{f[x,y], g[x,y]<=0}, {{x,x0},{y,y0}}]`
- **Métodos:**
  - Newton, Quasi-Newton (BFGS, L-BFGS)
  - Conjugate Gradient
  - Interior Point
  - Levenberg-Marquardt
  - Simulated Annealing
  - Differential Evolution
  - Nelder-Mead

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

#### 4.3 Optimización Convexa
- **Funciones convexas:** Detección automática
- **Second-order cone programming (SOCP)**
- **Semidefinite programming (SDP)**
- **Quadratic programming:** `QuadraticOptimization`
- **Conic optimization:** Generalized cone programs

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

#### 4.4 Optimización Global
- **Global search:** `NMinimize` con método "DifferentialEvolution"
- **Algoritmos genéticos:** `NMinimize[f, vars, Method->"GeneticAlgorithm"]`
- **Simulated annealing:** Búsqueda estocástica
- **Multi-start methods:** Múltiples inicializaciones

**Achronyme Priority:** ⭐⭐⭐ (Media)

#### 4.5 Optimización de Funcionales
- **Cálculo variacional:** `VariationalD`, `EulerEquations`
- **Control óptimo:** Ecuaciones de Hamilton-Jacobi-Bellman
- **Optimización de trayectorias**

**Achronyme Priority:** ⭐⭐ (Baja - muy avanzado)

---

### 5. 🌊 PROCESAMIENTO DE SEÑALES (DSP)

#### 5.1 Transformadas
- **Fourier Transform:**
  - Continua: `FourierTransform[f[t], t, ω]`
  - Discreta: `Fourier[{a,b,c,...}]` (DFT)
  - Rápida: `Fourier` usa FFT automáticamente
  - Inversa: `InverseFourierTransform`, `InverseFourier`
- **Laplace Transform:**
  - `LaplaceTransform[f[t], t, s]`
  - `InverseLaplaceTransform`
- **Z-Transform:**
  - `ZTransform[f[n], n, z]`
  - `InverseZTransform`
- **Wavelet Transform:**
  - Continua: `ContinuousWaveletTransform`
  - Discreta: `DiscreteWaveletTransform`
  - Packet: `WaveletPacketTransform`
- **Hilbert Transform:** `HilbertTransform`
- **Hankel Transform:** `HankelTransform`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO - tu implementación actual)

#### 5.2 Filtros
- **Diseño de filtros:**
  - FIR: `ToDiscreteTimeModel`, ventanas (Hamming, Hann, Blackman)
  - IIR: Butterworth, Chebyshev, Elliptic, Bessel
- **Aplicación:** `ListConvolve`, `ListCorrelate`
- **Respuesta en frecuencia:** `FrequencyResponse`, `BodePlot`
- **Filtrado:** `LowpassFilter`, `HighpassFilter`, `BandpassFilter`, `BandstopFilter`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

#### 5.3 Análisis Espectral
- **Periodograma:** `Periodogram`
- **Espectrograma:** `Spectrogram`
- **Densidad espectral de potencia:** `PowerSpectralDensity`
- **Ventanas:** Hamming, Hann, Blackman, Kaiser, etc.

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

#### 5.4 Convolución y Correlación
- **Convolución:** `Convolve[f, g, t, τ]`, `ListConvolve`
- **Correlación:** `Correlate`, `ListCorrelate`
- **Cross-correlation:** `CrossCorrelationFunction`
- **Autocorrelación:** `AutocorrelationFunction`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta)

---

### 6. 📊 FUNCIONES ESPECIALES

#### 6.1 Funciones Elementales
- **Exponencial/Logaritmos:** `Exp`, `Log`, `Log10`, `Log2`
- **Trigonométricas:** `Sin`, `Cos`, `Tan`, `Cot`, `Sec`, `Csc`
- **Trigonométricas inversas:** `ArcSin`, `ArcCos`, `ArcTan`, `ArcTan2`
- **Hiperbólicas:** `Sinh`, `Cosh`, `Tanh`, `Coth`, `Sech`, `Csch`
- **Hiperbólicas inversas:** `ArcSinh`, `ArcCosh`, `ArcTanh`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO)

#### 6.2 Funciones Especiales Clásicas
- **Gamma:** `Gamma[z]`, `LogGamma`, `PolyGamma`, `Beta`
- **Error functions:** `Erf[z]`, `Erfc`, `Erfi`
- **Bessel:** `BesselJ`, `BesselY`, `BesselI`, `BesselK`
- **Airy:** `AiryAi`, `AiryBi`, `AiryAiPrime`, `AiryBiPrime`
- **Legendre:** `LegendreP`, `LegendreQ`, `AssociatedLegendreP`
- **Hermite:** `HermiteH`
- **Laguerre:** `LaguerreL`, `AssociatedLaguerreL`
- **Chebyshev:** `ChebyshevT`, `ChebyshevU`
- **Elípticas:** `EllipticK`, `EllipticE`, `EllipticPi`, `JacobiSN`, `WeierstrassP`
- **Hypergeométricas:** `Hypergeometric0F1`, `Hypergeometric1F1`, `Hypergeometric2F1`
- **Zeta:** `Zeta[s]`, `RiemannSiegelZ`, `DirichletL`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta - competitividad académica)

#### 6.3 Funciones Generalizadas
- **Dirac delta:** `DiracDelta[x]`
- **Heaviside:** `HeavisideTheta[x]`
- **UnitStep:** `UnitStep[x]`
- **Sign:** `Sign[x]`
- **Abs:** `Abs[x]`
- **Piecewise:** `Piecewise[{{expr1, cond1}, {expr2, cond2}}]`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO - funciones por partes)

---

### 7. 🔢 TEORÍA DE NÚMEROS

#### 7.1 Funciones Aritméticas
- **Primalidad:** `PrimeQ[n]`, `NextPrime`, `PrimePi`
- **Factorización:** `FactorInteger[n]`, `PrimeFactors`
- **Divisores:** `Divisors[n]`, `DivisorSigma`
- **Euler phi:** `EulerPhi[n]`
- **Möbius:** `MoebiusMu[n]`
- **GCD/LCM:** `GCD[a,b]`, `LCM[a,b]`, `ExtendedGCD`

**Achronyme Priority:** ⭐⭐⭐ (Media)

#### 7.2 Aritmética Modular
- **Módulo:** `Mod[a, m]`
- **Power mod:** `PowerMod[a, b, m]`
- **Inverso modular:** `PowerMod[a, -1, m]`
- **Sistema de congruencias:** `ChineseRemainder`

**Achronyme Priority:** ⭐⭐⭐ (Media)

---

### 8. 📈 ESTADÍSTICA Y PROBABILIDAD

#### 8.1 Distribuciones
- **Continuas:** `NormalDistribution`, `UniformDistribution`, `ExponentialDistribution`, `ChiSquareDistribution`, etc.
- **Discretas:** `BinomialDistribution`, `PoissonDistribution`, `GeometricDistribution`
- **Multivariadas:** `MultinormalDistribution`, `MultivariateTPDistribution`
- **Funciones:** `PDF`, `CDF`, `Quantile`, `Mean`, `Variance`, `StandardDeviation`

**Achronyme Priority:** ⭐⭐⭐ (Media)

#### 8.2 Estadística Descriptiva
- **Centralidad:** `Mean`, `Median`, `Mode`
- **Dispersión:** `Variance`, `StandardDeviation`, `InterquartileRange`
- **Correlación:** `Correlation`, `Covariance`

**Achronyme Priority:** ⭐⭐⭐ (Media)

#### 8.3 Test de Hipótesis
- **t-test:** `TTest`
- **Chi-square:** `ChiSquareTest`
- **ANOVA:** `VarianceTest`

**Achronyme Priority:** ⭐⭐ (Baja)

---

### 9. 🎨 GEOMETRÍA Y VISUALIZACIÓN

#### 9.1 Geometría Computacional
- **Primitivas 2D:** `Point`, `Line`, `Circle`, `Polygon`, `Rectangle`
- **Primitivas 3D:** `Sphere`, `Cube`, `Cylinder`, `Cone`
- **Regiones:** `Region`, `RegionIntersection`, `RegionUnion`
- **Medidas:** `Area`, `Volume`, `ArcLength`, `Perimeter`
- **Transformaciones:** `Translate`, `Rotate`, `Scale`, `Reflect`

**Achronyme Priority:** ⭐⭐⭐ (Media - diferenciador visual)

#### 9.2 Plotting 2D
- **Funciones:** `Plot[f[x], {x, a, b}]`
- **Paramétrico:** `ParametricPlot`
- **Polar:** `PolarPlot`
- **Implícito:** `ContourPlot`, `RegionPlot`
- **Datos:** `ListPlot`, `ListLinePlot`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta - UX crítica)

#### 9.3 Plotting 3D
- **Superficies:** `Plot3D[f[x,y], {x,a,b}, {y,c,d}]`
- **Paramétrico:** `ParametricPlot3D`
- **Contornos:** `ContourPlot3D`
- **Densidad:** `DensityPlot3D`
- **Vectores:** `VectorPlot3D`, `StreamPlot3D`

**Achronyme Priority:** ⭐⭐⭐ (Media - bueno tener)

---

### 10. 💻 COMPUTACIÓN SIMBÓLICA AVANZADA

#### 10.1 Manipulación de Expresiones
- **Pattern matching:** Reglas de transformación
- **Reemplazos:** `ReplaceAll`, `ReplaceRepeated`
- **Assumptions:** `Assuming[cond, expr]`
- **Simplificación condicional:** `Refine[expr, assumptions]`

**Achronyme Priority:** ⭐⭐⭐⭐⭐ (CRÍTICO - motor CAS)

#### 10.2 Programación Funcional
- **Map/Apply:** `Map[f, list]`, `Apply[f, expr]`
- **Pure functions:** `Function[x, x^2]`, `#^2 &`
- **Fold:** `Fold[f, init, list]`
- **Nest:** `Nest[f, expr, n]`

**Achronyme Priority:** ⭐⭐⭐⭐ (Alta - flexibilidad)

---

## 🎯 MATRIZ DE PRIORIDADES PARA ACHRONYME

### Fase 1 - MVP (Sprints 1-4): Core CAS
| Feature | Priority | Status |
|---------|----------|--------|
| Parser de expresiones simbólicas | ⭐⭐⭐⭐⭐ | Pendiente |
| Álgebra elemental (simplify, expand, factor) | ⭐⭐⭐⭐⭐ | Pendiente |
| Derivación simbólica | ⭐⭐⭐⭐⭐ | Pendiente |
| Integración numérica | ⭐⭐⭐⭐⭐ | Parcial (Simpson) |
| Ecuaciones lineales (Simplex) | ⭐⭐⭐⭐⭐ | Pendiente |
| DFT/FFT | ⭐⭐⭐⭐⭐ | Parcial (JS) |
| Funciones elementales | ⭐⭐⭐⭐⭐ | Pendiente |

### Fase 2 - Advanced Math (Sprints 5-8)
| Feature | Priority | Status |
|---------|----------|--------|
| Integración simbólica | ⭐⭐⭐⭐ | Pendiente |
| Resolución de ODEs | ⭐⭐⭐⭐⭐ | Pendiente |
| Álgebra lineal completa | ⭐⭐⭐⭐ | Pendiente |
| Funciones especiales (Bessel, etc) | ⭐⭐⭐⭐ | Pendiente |
| Transformadas (Laplace, Z) | ⭐⭐⭐⭐ | Pendiente |
| Filtros digitales (FIR, IIR) | ⭐⭐⭐⭐ | Pendiente |

### Fase 3 - Professional (Sprints 9-12)
| Feature | Priority | Status |
|---------|----------|--------|
| PDEs numéricas (FEM) | ⭐⭐⭐⭐ | Pendiente |
| Optimización no lineal | ⭐⭐⭐⭐ | Pendiente |
| Plotting 2D/3D | ⭐⭐⭐⭐ | Pendiente |
| Estadística avanzada | ⭐⭐⭐ | Pendiente |
| Geometría computacional | ⭐⭐⭐ | Pendiente |

---

## 📊 VENTAJAS COMPETITIVAS DE ACHRONYME

### ✅ Ventajas sobre Mathematica

1. **Open Source:** Código abierto, comunidad puede contribuir
2. **Web-first:** Corre en navegador sin instalación
3. **Performance:** WASM puede ser 10-20x más rápido en operaciones numéricas
4. **Tamaño:** <5MB vs 7GB de Mathematica
5. **Gratis:** Sin costos de licencia
6. **API REST:** Puede usarse como servicio backend
7. **Modular:** Lazy loading de features según necesidad
8. **Cross-platform:** Mismo código en web, Node.js, desktop (Electron)

### ⚠️ Desventajas (a mitigar)

1. **Cantidad de features:** 6,600 funciones vs ~2,000 target
2. **Integración simbólica:** Muy complejo (puede usar CAS externo)
3. **UI/Notebooks:** Mathematica tiene notebooks muy maduros
4. **Documentación:** 35+ años de docs vs nuevo proyecto
5. **Ecosistema:** Wolfram tiene datasets, curated data, etc.

---

## 🚀 ROADMAP PROPUESTO

### Q1 2026: Core CAS (Target: 500 funciones)
- Parser + Compiler + Evaluator
- Álgebra simbólica básica
- Cálculo diferencial/integral numérico
- DSP (DFT, FFT, convolución)
- Linear programming (Simplex)

### Q2 2026: Advanced Math (Target: 1,000 funciones)
- Integración simbólica (limitada)
- ODE solver (simbólico + numérico)
- Álgebra lineal completa
- Funciones especiales
- Transformadas (Laplace, Z)

### Q3 2026: Professional Tools (Target: 1,500 funciones)
- PDE solver numérico
- Optimización no lineal
- Plotting engine (2D/3D)
- Estadística avanzada
- Exportación a código (Python, C++, JS)

### Q4 2026: Ecosystem (Target: 2,000 funciones)
- Notebook interface (Jupyter-like)
- Cloud compute backend
- Mobile apps
- Plugin system
- Marketplace de extensiones

---

## 📚 REFERENCIAS

- [Wolfram Language Documentation](https://reference.wolfram.com/language/)
- [Mathematica Feature Comparison](https://www.wolfram.com/mathematica/compare-mathematica/)
- [Version 14.3 Release Notes](https://www.wolfram.com/mathematica/quick-revision-history/)
- [Symbolic Computation Blog](https://blog.wolfram.com/category/technology/symbolic-computation/)

---

**Documento creado:** 2025-10-05
**Última actualización:** 2025-10-05
**Autor:** @eddndev + Claude Code
**Propósito:** Guía definitiva para diseñar Achronyme como competidor open-source de Mathematica
