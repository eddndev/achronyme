# 📦 Template de Metodología AGENTS.md

Este directorio contiene la **plantilla base** para aplicar la metodología AGENTS.md a cualquier proyecto.

## 📁 Contenido

```
docs-example/
├── AGENTS.md                          # Metodología completa (copiar a docs/)
├── workflow/                          # Documentación de procesos
│   ├── 01-team-workflow.md           # Flujo de trabajo en equipo
│   ├── 02-branch-protection.md       # Protección de rama main
│   └── 03-github-projects-setup.md   # Setup de GitHub Projects
├── .github/
│   ├── workflows/
│   │   └── project-board-automation.yml  # ⚠️ TEMPLATE - Configurar IDs
│   ├── ISSUE_TEMPLATE/
│   │   ├── feature.yml               # Template para features
│   │   ├── bug.yml                   # Template para bugs
│   │   ├── chore.yml                 # Template para chores
│   │   └── config.yml                # ⚠️ TEMPLATE - Actualizar URLs
│   ├── CODEOWNERS                    # Asignación de reviewers
│   └── pull_request_template.md      # Template de PRs
└── README.md                          # Este archivo
```

---

## 🚀 Cómo Usar Este Template

### 1. Copiar a Nuevo Proyecto

```bash
# Desde el directorio commons/ (o donde guardes este template)
cp -r docs-example/AGENTS.md [NUEVO_PROYECTO]/docs/
cp -r docs-example/workflow/ [NUEVO_PROYECTO]/docs/
cp -r docs-example/.github/ [NUEVO_PROYECTO]/
```

### 2. Configurar Archivos con Placeholders

#### A. Workflow de Project Board

**Archivo:** `.github/workflows/project-board-automation.yml`

1. Crear Personal Access Token (PAT):
   - Ir a: https://github.com/settings/tokens
   - Scopes: `repo` + `project`
   - Agregar como secret `PROJECT_PAT` en el repositorio

2. Obtener IDs del proyecto:
   ```bash
   # Listar proyectos
   gh project list --owner [OWNER]

   # Ver campos del proyecto
   gh project field-list [PROJECT_NUMBER] --owner [OWNER]
   ```

3. Reemplazar en el archivo:
   - `YOUR_PROJECT_ID_HERE` → ID del proyecto
   - `YOUR_STATUS_FIELD_ID_HERE` → ID del campo Status
   - `YOUR_TODO_OPTION_ID_HERE` → ID de opción "Todo"
   - `YOUR_IN_PROGRESS_ID` → ID de opción "In Progress"
   - `YOUR_DONE_OPTION_ID_HERE` → ID de opción "Done"
   - `[OWNER]` (línea 67) → Tu username u organización

#### B. Issue Template Config

**Archivo:** `.github/ISSUE_TEMPLATE/config.yml`

Reemplazar:
- `[OWNER]` → Tu username u organización
- `[REPO]` → Nombre del repositorio

#### C. CODEOWNERS

**Archivo:** `.github/CODEOWNERS`

Ya está configurado con `@eddndev`. Si es otro usuario, buscar/reemplazar.

#### D. Pull Request Template

**Archivo:** `.github/pull_request_template.md`

Ya está configurado con `@eddndev`. Si es otro usuario, buscar/reemplazar.

---

## 📋 Checklist de Setup en Nuevo Proyecto

### Fase 1: Documentación Base
- [ ] Copiar `AGENTS.md` a `docs/`
- [ ] Copiar `workflow/` a `docs/`
- [ ] Leer y entender toda la metodología

### Fase 2: Configuración GitHub
- [ ] Copiar `.github/` al proyecto
- [ ] Crear Personal Access Token (PAT)
- [ ] Agregar PAT como secret `PROJECT_PAT`
- [ ] Crear GitHub Project v2
- [ ] Obtener todos los IDs necesarios
- [ ] Configurar `project-board-automation.yml` con IDs reales
- [ ] Actualizar `config.yml` con URLs correctas
- [ ] Verificar CODEOWNERS tiene el username correcto

### Fase 3: Protección de Ramas
- [ ] Seguir `docs/workflow/02-branch-protection.md`
- [ ] Configurar protección en rama `main`
- [ ] Verificar que push directo está bloqueado

### Fase 4: Pruebas
- [ ] Crear issue de prueba
- [ ] Verificar que se mueve a "Todo" automáticamente
- [ ] Asignarte la issue
- [ ] Verificar que se mueve a "In Progress"
- [ ] Cerrar la issue
- [ ] Verificar que se mueve a "Done"

### Fase 5: Equipo (si aplica)
- [ ] Compartir `docs/AGENTS.md` con el equipo
- [ ] Compartir `docs/workflow/01-team-workflow.md`
- [ ] Hacer onboarding siguiendo checklist en workflow

---

## 🔄 Actualizar Template

Cuando hagas mejoras a la metodología en un proyecto:

1. Actualizar `docs-example/` con los cambios
2. Documentar cambios en un changelog (opcional)
3. Aplicar a otros proyectos activos si es necesario

---

## 📚 Documentación de Referencia

- **Metodología completa:** `AGENTS.md`
- **Workflow de equipo:** `workflow/01-team-workflow.md`
- **Branch protection:** `workflow/02-branch-protection.md`
- **GitHub Projects:** `workflow/03-github-projects-setup.md`

---

## ⚠️ Notas Importantes

1. **NO commitear** este directorio `docs-example/` en proyectos reales
2. **Mantenerlo** solo en un repositorio `commons/` o similar
3. **Siempre configurar** los placeholders antes de usar
4. **Probar** el workflow completo antes de trabajar en equipo

---

**Template version:** 1.0
**Última actualización:** 2025-10-05
**Mantenido por:** @eddndev
