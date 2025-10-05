# User Stories - Achronyme

**Versión:** 1.0
**Fecha:** 2025-10-05
**Mantenido por:** @eddndev

---

## 1. Épicas del Proyecto

### Épica 1: Autenticación y Gestión de Usuarios
**Objetivo:** Permitir a los usuarios crear cuentas, autenticarse y gestionar sus perfiles.

### Épica 2: Herramientas de Procesamiento Digital de Señales (DSP)
**Objetivo:** Proporcionar herramientas interactivas para análisis de señales.

### Épica 3: Gestión de Proyectos/Assets
**Objetivo:** Permitir guardar, compartir y reutilizar cálculos y proyectos.

### Épica 4: Migración a WebAssembly
**Objetivo:** Mejorar rendimiento migrando operaciones matemáticas críticas a WASM con C++.

### Épica 5: Herramientas de Programación Lineal
**Objetivo:** Implementar solucionadores de problemas de optimización lineal.

### Épica 6: Cursos y Contenido Educativo
**Objetivo:** Proporcionar tutoriales interactivos y cursos gratuitos.

---

## 2. Historias de Usuario - Épica 1: Autenticación

### US-1.1: Registro de Usuario con Email
**Como** visitante del sitio
**Quiero** poder registrarme con mi email y contraseña
**Para** crear una cuenta y acceder a las herramientas

**Criterios de Aceptación:**
- [ ] Formulario de registro con campos: nombre, email, contraseña, confirmar contraseña
- [ ] Validación de email único en la base de datos
- [ ] Validación de contraseña (mínimo 8 caracteres)
- [ ] Envío de email de verificación
- [ ] Redirección al dashboard después del registro exitoso

**Prioridad:** Alta

---

### US-1.2: Login con Email y Contraseña
**Como** usuario registrado
**Quiero** poder iniciar sesión con mi email y contraseña
**Para** acceder a mis proyectos guardados

**Criterios de Aceptación:**
- [ ] Formulario de login con email y contraseña
- [ ] Opción "Recordarme" (remember me)
- [ ] Validación de credenciales
- [ ] Redirección al dashboard si las credenciales son correctas
- [ ] Mensaje de error si las credenciales son incorrectas

**Prioridad:** Alta

---

### US-1.3: Login con Google OAuth
**Como** visitante del sitio
**Quiero** poder iniciar sesión con mi cuenta de Google
**Para** evitar crear una nueva contraseña

**Criterios de Aceptación:**
- [ ] Botón "Continuar con Google" en página de login y registro
- [ ] Integración con Google OAuth 2.0
- [ ] Creación automática de cuenta si es nuevo usuario
- [ ] Login automático si la cuenta ya existe
- [ ] Almacenar google_id y tokens en la BD

**Prioridad:** Media

---

### US-1.4: Recuperación de Contraseña
**Como** usuario registrado
**Quiero** poder recuperar mi contraseña si la olvido
**Para** volver a acceder a mi cuenta

**Criterios de Aceptación:**
- [ ] Link "¿Olvidaste tu contraseña?" en página de login
- [ ] Formulario para ingresar email
- [ ] Envío de email con link de reseteo
- [ ] Página para establecer nueva contraseña
- [ ] Expiración del link después de 1 hora

**Prioridad:** Media

---

### US-1.5: Autenticación de Dos Factores (2FA)
**Como** usuario preocupado por la seguridad
**Quiero** habilitar autenticación de dos factores
**Para** proteger mi cuenta con una capa extra de seguridad

**Criterios de Aceptación:**
- [ ] Opción en configuración de cuenta para habilitar 2FA
- [ ] Generación de QR code para apps de autenticación (Google Authenticator, Authy)
- [ ] Solicitar código 2FA durante login si está habilitado
- [ ] Códigos de recuperación descargables
- [ ] Opción para deshabilitar 2FA

**Prioridad:** Baja

---

## 3. Historias de Usuario - Épica 2: DSP Tools

### US-2.1: Calcular Series de Fourier
**Como** estudiante de ingeniería
**Quiero** calcular y visualizar la serie de Fourier de una función
**Para** entender la descomposición en frecuencias

**Criterios de Aceptación:**
- [ ] Input para función f(x) en notación matemática
- [ ] Input para número de armónicos (n)
- [ ] Input para período T
- [ ] Botón "Calcular" que compute los coeficientes a₀, aₙ, bₙ
- [ ] Visualización gráfica de la función original vs. aproximación
- [ ] Display de fórmula LaTeX de la serie resultante
- [ ] Tabla con valores de coeficientes

**Prioridad:** Alta (✅ Ya implementado)

---

### US-2.2: Calcular Transformada de Fourier
**Como** ingeniero
**Quiero** calcular la transformada de Fourier de una señal
**Para** analizar su contenido frecuencial

**Criterios de Aceptación:**
- [ ] Input para señal en el dominio del tiempo
- [ ] Opción de señal discreta (DFT) o continua (FT)
- [ ] Cálculo de magnitud y fase
- [ ] Gráfica de magnitud vs. frecuencia
- [ ] Gráfica de fase vs. frecuencia
- [ ] Opción de exportar datos

**Prioridad:** Alta (✅ Ya implementado)

---

### US-2.3: Visualizar Convolución de Señales
**Como** estudiante
**Quiero** visualizar la convolución de dos señales
**Para** entender el proceso gráficamente

**Criterios de Aceptación:**
- [ ] Input para señal x(t)
- [ ] Input para señal h(t) (respuesta al impulso)
- [ ] Animación paso a paso de la convolución
- [ ] Gráfica de resultado y(t) = x(t) * h(t)
- [ ] Controles de play/pause/step en la animación

**Prioridad:** Alta (✅ Ya implementado)

---

## 4. Historias de Usuario - Épica 3: Gestión de Assets

### US-3.1: Guardar Cálculo como Proyecto
**Como** usuario autenticado
**Quiero** guardar mis cálculos como proyectos
**Para** poder volver a ellos más tarde

**Criterios de Aceptación:**
- [ ] Botón "Guardar proyecto" en cada herramienta
- [ ] Modal para ingresar nombre y descripción
- [ ] Guardar inputs, resultados y configuración en JSON
- [ ] Confirmación de guardado exitoso
- [ ] Aparece en "Mis Proyectos"

**Prioridad:** Alta

---

### US-3.2: Cargar Proyecto Guardado
**Como** usuario autenticado
**Quiero** cargar un proyecto que guardé anteriormente
**Para** continuar trabajando en él

**Criterios de Aceptación:**
- [ ] Sección "Mis Proyectos" en el dashboard
- [ ] Lista de proyectos con nombre, tipo, y fecha
- [ ] Click en proyecto carga la herramienta con datos guardados
- [ ] Restauración completa del estado (inputs, gráficas, resultados)

**Prioridad:** Alta

---

### US-3.3: Compartir Proyecto Públicamente
**Como** usuario autenticado
**Quiero** hacer público un proyecto
**Para** compartirlo con otros usuarios o en redes sociales

**Criterios de Aceptación:**
- [ ] Toggle "Público/Privado" en configuración del proyecto
- [ ] Generación de URL única y amigable (slug)
- [ ] Página pública que muestra el proyecto (read-only)
- [ ] Botón "Copiar enlace" para compartir
- [ ] Contador de vistas

**Prioridad:** Media

---

### US-3.4: Eliminar Proyecto
**Como** usuario autenticado
**Quiero** eliminar un proyecto
**Para** mantener organizada mi lista de proyectos

**Criterios de Aceptación:**
- [ ] Botón "Eliminar" en cada proyecto
- [ ] Modal de confirmación "¿Estás seguro?"
- [ ] Eliminación de la base de datos
- [ ] Mensaje de confirmación
- [ ] Actualización de la lista sin recargar página

**Prioridad:** Media

---

## 5. Historias de Usuario - Épica 4: WebAssembly Migration

### US-4.1: Migrar Cálculo de Fourier a WASM
**Como** desarrollador
**Quiero** migrar el cálculo de series de Fourier a WebAssembly
**Para** mejorar el rendimiento en funciones complejas

**Criterios de Aceptación:**
- [ ] Implementar algoritmo de Fourier en C++
- [ ] Compilar a WASM con Emscripten
- [ ] Crear binding JavaScript para llamar desde el frontend
- [ ] Benchmark: tiempo de cálculo <50ms para funciones estándar
- [ ] Fallback a Math.js si WASM no está soportado
- [ ] Tests de paridad (mismo resultado que Math.js)

**Prioridad:** Alta

---

### US-4.2: Migrar FFT (Fast Fourier Transform) a WASM
**Como** desarrollador
**Quiero** implementar FFT en C++ compilado a WASM
**Para** procesar señales grandes de forma eficiente

**Criterios de Aceptación:**
- [ ] Implementar algoritmo Cooley-Tukey en C++
- [ ] Soporte para señales de hasta 1M de puntos
- [ ] Benchmark: 10x más rápido que implementación JS
- [ ] Manejo de memoria eficiente (sin leaks)
- [ ] Tests de correctitud vs. implementación de referencia

**Prioridad:** Alta

---

### US-4.3: Crear Módulo WASM para Álgebra Lineal
**Como** desarrollador
**Quiero** un módulo WASM para operaciones matriciales
**Para** futuras herramientas de álgebra lineal

**Criterios de Aceptación:**
- [ ] Implementar multiplicación de matrices
- [ ] Implementar inversión de matrices
- [ ] Implementar determinante
- [ ] Implementar descomposición LU
- [ ] Soporte para matrices de hasta 1000x1000
- [ ] API JavaScript bien documentada

**Prioridad:** Media

---

## 6. Historias de Usuario - Épica 5: Linear Programming

### US-5.1: Resolver Problema de Programación Lineal (Método Simplex)
**Como** estudiante de investigación de operaciones
**Quiero** resolver problemas de programación lineal con el método simplex
**Para** encontrar la solución óptima

**Criterios de Aceptación:**
- [ ] Input para función objetivo (maximizar/minimizar)
- [ ] Input para restricciones (≤, ≥, =)
- [ ] Input para variables no negativas
- [ ] Algoritmo Simplex implementado
- [ ] Mostrar tabla simplex paso a paso
- [ ] Mostrar solución óptima y valor de Z
- [ ] Detectar casos sin solución o infinitas soluciones

**Prioridad:** Media

---

### US-5.2: Visualizar Región Factible (2D)
**Como** estudiante
**Quiero** visualizar la región factible de un problema 2D
**Para** entender geométricamente el problema

**Criterios de Aceptación:**
- [ ] Gráfica 2D de las restricciones
- [ ] Sombreado de región factible
- [ ] Marcado de puntos esquina (vértices)
- [ ] Línea de isovalor de la función objetivo
- [ ] Marcado del punto óptimo
- [ ] Solo para problemas con 2 variables

**Prioridad:** Baja

---

## 7. Historias de Usuario - Épica 6: Contenido Educativo

### US-6.1: Ver Tutorial Interactivo de Series de Fourier
**Como** estudiante nuevo en DSP
**Quiero** ver un tutorial paso a paso de series de Fourier
**Para** aprender cómo usar la herramienta

**Criterios de Aceptación:**
- [ ] Tutorial dividido en pasos claros
- [ ] Ejemplos interactivos (pueden modificarse)
- [ ] Explicación de cada coeficiente (a₀, aₙ, bₙ)
- [ ] Ejercicios prácticos
- [ ] Botón "Siguiente" y "Anterior"
- [ ] Progreso guardado (si está autenticado)

**Prioridad:** Media

---

### US-6.2: Acceder a Biblioteca de Ejemplos
**Como** usuario
**Quiero** acceder a una biblioteca de ejemplos pre-definidos
**Para** aprender de casos comunes

**Criterios de Aceptación:**
- [ ] Categorías: Señales básicas, Filtros, Sistemas, etc.
- [ ] Preview de cada ejemplo
- [ ] Botón "Cargar en herramienta"
- [ ] Descripción de cada ejemplo
- [ ] Búsqueda por nombre o categoría

**Prioridad:** Baja

---

## 8. Backlog de Funcionalidades Futuras

### Fase 2 (Q2-Q3 2025)
- [ ] US-7.1: Herramientas de Estadística Básica
- [ ] US-7.2: Calculadora de Probabilidades
- [ ] US-8.1: Exportar resultados a PDF
- [ ] US-8.2: Exportar datos a CSV/Excel
- [ ] US-9.1: Multi-idioma (Español, Inglés, Portugués, Francés)

### Fase 3 (Q4 2025)
- [ ] US-10.1: Modo oscuro (Dark mode)
- [ ] US-10.2: PWA con soporte offline
- [ ] US-11.1: API pública REST
- [ ] US-11.2: Webhooks para integraciones

### Fase 4 (2026+)
- [ ] US-12.1: Sistema de plugins/extensiones
- [ ] US-13.1: App móvil nativa (React Native)
- [ ] US-14.1: Colaboración en tiempo real
- [ ] US-14.2: Chat en vivo entre usuarios

---

## 9. Criterios de Aceptación Globales

**Para todas las historias de usuario, se debe cumplir:**

### Funcionalidad
- Los cálculos deben ser matemáticamente correctos
- Manejo de errores y casos edge
- Performance aceptable (según benchmarks)

### UI/UX
- Responsive en mobile, tablet y desktop
- Accesible (WCAG 2.1 Level AA)
- Feedback visual inmediato (loaders, confirmaciones)
- Mensajes de error claros y útiles

### Testing
- Tests unitarios con >80% coverage
- Tests de integración para flujos críticos
- Tests E2E para user journeys principales

### Documentación
- Código documentado (PHPDoc, JSDoc)
- README actualizado si es necesario
- Actualizar docs técnicos en `/docs/`

---

**Documento vivo - Actualizado con cada nuevo feature o cambio de prioridades**