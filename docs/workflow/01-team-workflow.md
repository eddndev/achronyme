# 👥 Workflow de Trabajo en Equipo

## 🌳 Estrategia de Ramas

### Modelo: GitHub Flow

- **`main`** → Rama protegida, solo acepta PRs aprobados
- **`feature/[issue-num]-[descripcion]`** → Para nuevas funcionalidades
- **`fix/[issue-num]-[descripcion]`** → Para corrección de bugs
- **`chore/[issue-num]-[descripcion]`** → Para tareas de mantenimiento

### Convención de Nombres de Rama

```bash
# Formato: [tipo]/issue-[NUM]-[descripcion-corta]

# Ejemplos:
feature/issue-3-tailwind-config
feature/issue-10-dark-mode-toggle
fix/issue-29-workflow-bug
chore/issue-8-database-seeders
```

---

## 🔒 Proteger la Rama `main`

**⚠️ CRÍTICO: Configurar protecciones ANTES de empezar el trabajo en equipo**

### Paso 1: Configurar Branch Protection Rules

1. **Ir a:** `https://github.com/[OWNER]/[REPO]/settings/branches`
2. **Click:** "Add branch protection rule"
3. **Branch name pattern:** `main`
4. **Activar estas reglas:**

```yaml
✅ Require a pull request before merging
   ✅ Require approvals: 1 (tú como reviewer)
   ✅ Dismiss stale pull request approvals when new commits are pushed
   ✅ Require review from Code Owners (opcional)

✅ Require status checks to pass before merging
   ✅ Require branches to be up to date before merging
   ✅ Status checks required:
      - Claude Code Review (si está configurado)
      - Tests (cuando los tengas)

✅ Require conversation resolution before merging

✅ Do not allow bypassing the above settings
   ⚠️ Incluir administradores (para que TÚ también sigas las reglas)

❌ Allow force pushes (NUNCA activar esto)
❌ Allow deletions (NUNCA activar esto)
```

5. **Click:** "Create" / "Save changes"

---

## 👨‍💻 Flujo de Trabajo del Desarrollador

### 1. Recibir Asignación de Issue

```bash
# El desarrollador recibe una issue asignada (ej: #3)
# La issue automáticamente se mueve a "In Progress" al asignarse
```

### 2. Crear Rama desde `main`

```bash
# Siempre partir de main actualizado
git checkout main
git pull origin main

# Crear rama feature
git checkout -b feature/issue-3-tailwind-config
```

### 3. Trabajar en la Issue

```bash
# Hacer commits siguiendo convenciones
git add .
git commit -m "feat(tailwind): configurar design tokens purple/blue

- Agregar colores purple (50-950) en tailwind.config.js
- Configurar fuentes Inter y JetBrains Mono
- Definir breakpoints responsive

Refs #3"

# Hacer push de la rama
git push -u origin feature/issue-3-tailwind-config
```

### 4. Crear Pull Request

```bash
# Opción 1: Desde CLI
gh pr create --title "[Setup] Configurar Tailwind con design tokens" \
  --body "Resuelve #3

## Cambios
- Configurados design tokens en tailwind.config.js
- Importadas fuentes Inter y JetBrains Mono
- Definidos breakpoints responsive

## Checklist
- [x] Todos los criterios de aceptación cumplidos
- [x] Código probado localmente
- [x] Sin errores en consola
- [ ] Requiere review de @eddndev" \
  --assignee "@me" \
  --label "Status: Needs Review"

# Opción 2: Desde la web
# GitHub te sugerirá crear un PR automáticamente
```

### 5. El Workflow Automático hace:

- ✅ Mueve la issue a estado "Needs Review" (por el label)
- ✅ Ejecuta workflows de CI (Claude Code Review, tests)
- ✅ Espera tu aprobación como Code Owner

---

## 👀 Tu Rol como Code Reviewer

### Checklist de Code Review

Cuando recibes un PR, verificar:

#### ✅ Funcionalidad
- [ ] Los cambios resuelven la issue completamente
- [ ] Se cumplen todos los criterios de aceptación
- [ ] No hay funcionalidad rota (side effects)

#### ✅ Calidad de Código
- [ ] Sigue las convenciones del proyecto
- [ ] Código limpio y legible
- [ ] No hay código duplicado
- [ ] Nombres de variables/funciones descriptivos

#### ✅ Testing
- [ ] Los cambios están probados
- [ ] No hay regresiones
- [ ] Funciona en light/dark mode (si aplica)
- [ ] Responsive en mobile/tablet/desktop (si aplica)

#### ✅ Documentación
- [ ] Código comentado donde es necesario
- [ ] README actualizado si aplica
- [ ] Docs técnicos actualizados si aplica

#### ✅ Git
- [ ] Commits son atómicos y descriptivos
- [ ] Rama está actualizada con `main`
- [ ] No hay conflictos de merge

### Comandos Útiles para Review

```bash
# Ver el PR
gh pr view [NUM]

# Checkout del PR localmente para probar
gh pr checkout [NUM]

# Ver cambios
gh pr diff [NUM]

# Aprobar el PR
gh pr review [NUM] --approve -b "LGTM! 🚀 Buen trabajo"

# Solicitar cambios
gh pr review [NUM] --request-changes -b "Por favor corregir:
- Falta manejar el caso edge X
- El nombre de la función debería ser más descriptivo"

# Comentar sin aprobar/rechazar
gh pr review [NUM] --comment -b "Pregunta: ¿Por qué usaste este approach?"

# Hacer merge (después de aprobar)
gh pr merge [NUM] --squash --delete-branch
```

---

## 🔄 Workflow Completo en Equipo

### Escenario Típico:

1. **Planificación del Sprint**
   - Tú creas issues y las agregas al proyecto
   - Asignas issues a desarrolladores
   - Issues se mueven a "In Progress" automáticamente

2. **Desarrollo**
   - Dev crea rama `feature/issue-X-descripcion`
   - Dev trabaja y hace commits
   - Dev hace push y abre PR

3. **Code Review (Tú)**
   - Recibes notificación del PR
   - Revisas código usando checklist
   - Apruebas o solicitas cambios

4. **Merge**
   - Si está aprobado: hacer merge (squash recomendado)
   - Issue se cierra automáticamente (por "Closes #X" en PR)
   - Issue se mueve a "Done" automáticamente

5. **Deploy**
   - `main` siempre está lista para deploy
   - CI/CD puede hacer deploy automático (opcional)

---

## 🚨 Reglas de Oro para el Equipo

### ❌ NUNCA hacer:
- Push directo a `main` (bloqueado por protecciones)
- Force push a ramas compartidas
- Merge sin aprobación
- Commits sin mensaje descriptivo
- Dejar PRs abiertos más de 24-48h

### ✅ SIEMPRE hacer:
- Crear rama desde `main` actualizado
- Usar convención de nombres de ramas
- Vincular PRs a issues con "Closes #X" o "Refs #X"
- Escribir mensajes de commit descriptivos
- Solicitar review antes de merge
- Eliminar rama después de merge

---

## 📋 CODEOWNERS (Opcional pero Recomendado)

Crea `.github/CODEOWNERS` para asignar reviewers automáticamente:

```
# El tech lead es el owner por defecto de todo
* @eddndev

# Áreas específicas (si tienes especialistas)
/docs/ @eddndev
/database/ @eddndev @[DB_SPECIALIST]
*.yml @eddndev

# Archivos críticos requieren review del tech lead
/docs/AGENTS.md @eddndev
/.github/workflows/ @eddndev
```

---

## 🎯 Convenciones de Commit

**Formato:** `tipo(scope): mensaje corto`

```bash
# Tipos:
feat:     Nueva funcionalidad
fix:      Corrección de bug
chore:    Tareas de mantenimiento
docs:     Documentación
style:    Formateo, no cambia lógica
refactor: Refactorización
test:     Agregar o modificar tests
perf:     Mejora de performance

# Ejemplos:
feat(auth): implementar login con Google
fix(navbar): corregir overflow en mobile
chore(deps): actualizar dependencias
docs(readme): agregar instrucciones de instalación
```

---

## 🔧 Comandos para el Team Lead (Tú)

```bash
# Ver todas las ramas activas
git branch -a

# Ver PRs abiertos
gh pr list

# Ver estado del proyecto
gh project view [PROJECT_NUMBER] --owner [OWNER]

# Ver quién está trabajando en qué
gh issue list --assignee="*"

# Cerrar sprint y crear retrospectiva
# (Manual: actualizar docs/sprints/XX-nombre.md)

# Mergear y cerrar PR
gh pr merge [NUM] --squash --delete-branch
```

---

## 📊 Métricas de Equipo (Opcional)

### KPIs a monitorear:

- **Cycle Time:** Tiempo de issue asignada → merged
- **PR Review Time:** Tiempo de PR abierto → aprobado
- **Bugs en Producción:** Issues con label `Type: Bug` post-merge
- **Code Review Feedback:** Número de comentarios por PR

```bash
# Ver métricas básicas
gh pr list --state merged --limit 20 --json number,title,createdAt,mergedAt
```

---

## 🎓 Onboarding de Nuevos Devs

### Checklist para nuevos desarrolladores:

1. [ ] Leer `docs/AGENTS.md` completo (metodología inquebrantable)
2. [ ] Configurar git local:
   ```bash
   git config user.name "Nombre"
   git config user.email "email@example.com"
   ```
3. [ ] Clonar repo y ejecutar setup:
   ```bash
   git clone https://github.com/[OWNER]/[REPO].git
   cd [REPO]
   composer install
   npm install
   ```
4. [ ] Leer `docs/workflow/01-team-workflow.md` (este archivo)
5. [ ] Practicar flujo con una issue de prueba `Priority: Low`
6. [ ] Hacer primer PR para review

---

## 🚀 Tips para Maximizar Productividad

### Para ti como Lead:
- Revisar PRs máximo 2 veces al día (mañana/tarde)
- Usar templates de PR para estandarizar
- Automatizar lo que se pueda (linters, tests, CI/CD)
- Hacer code reviews constructivas, no solo críticas

### Para el equipo:
- PRs pequeños (<300 líneas) se revisan más rápido
- Self-review antes de solicitar review
- Usar draft PRs para trabajo en progreso
- Comunicar bloqueos inmediatamente

---

**Documentación relacionada:**
- Metodología completa: `docs/AGENTS.md`
- Setup de Projects: `docs/workflow/03-github-projects-setup.md`
- Protección de rama main: `docs/workflow/02-branch-protection.md`
