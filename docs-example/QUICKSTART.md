# ⚡ QUICKSTART - Setup Rápido

Guía ultra-rápida para aplicar la metodología AGENTS.md a un nuevo proyecto.

---

## 🎯 En 5 Minutos

### 1. Copiar archivos (1 min)

```bash
# Desde tu carpeta commons/
cd /path/to/commons/docs-example

# Copiar al nuevo proyecto
cp AGENTS.md /path/to/nuevo-proyecto/docs/
cp -r workflow/ /path/to/nuevo-proyecto/docs/
cp -r .github/ /path/to/nuevo-proyecto/
```

### 2. Crear PAT y Project (2 min)

```bash
# Crear PAT en: https://github.com/settings/tokens
# Scopes: repo + project
# Copiar el token

# Agregar como secret
# Ir a: https://github.com/[OWNER]/[REPO]/settings/secrets/actions
# Name: PROJECT_PAT
# Value: [pegar token]

# Crear proyecto
gh project create --owner [OWNER] --title "[NOMBRE] - Development"
# Copiar el PROJECT_ID que aparece en el output
```

### 3. Obtener IDs (1 min)

```bash
# Ver proyectos
gh project list --owner [OWNER]
# Anotar PROJECT_NUMBER

# Ver campos
gh project field-list [PROJECT_NUMBER] --owner [OWNER]
# Anotar STATUS_FIELD_ID y los OPTION_IDs (Todo, In Progress, Done)
```

### 4. Configurar workflow (1 min)

Editar `.github/workflows/project-board-automation.yml`:

```yaml
env:
  PROJECT_ID: PVT_kwHO...          # ← Pegar aquí
  STATUS_FIELD_ID: PVTSSF_lAHO...  # ← Pegar aquí
  TODO_OPTION_ID: f75ad846         # ← Pegar aquí
  IN_PROGRESS_OPTION_ID: 47fc9ee4  # ← Pegar aquí
  DONE_OPTION_ID: 98236657         # ← Pegar aquí
```

Y en línea 67:
```yaml
gh project item-add ... --owner eddndev  # ← Cambiar a tu username
```

### 5. Configurar URLs (30 seg)

Editar `.github/ISSUE_TEMPLATE/config.yml`:

```yaml
url: https://github.com/eddndev/mi-proyecto/...  # ← Cambiar
```

---

## ✅ Verificar que Funciona

```bash
# Crear issue de prueba
gh issue create --title "Test automation" --body "Testing"

# Ver si se agregó al proyecto
gh project item-list [PROJECT_NUMBER] --owner [OWNER]

# Asignarte
gh issue edit [NUM] --add-assignee "@me"

# Ver si se movió a "In Progress"
# (Verificar en el proyecto web o con gh project item-list)

# Cerrar
gh issue close [NUM]

# Ver si se movió a "Done"
```

Si todo se movió correctamente: **✅ LISTO**

---

## 🔒 Proteger rama main

```bash
# Ir a: https://github.com/[OWNER]/[REPO]/settings/branches
# Click "Add branch protection rule"
# Pattern: main
# Activar:
#   ☑️ Require a pull request before merging
#   ☑️ Require approvals: 1
#   ☑️ Include administrators
# Click "Create"
```

Verificar:
```bash
# Esto debería fallar:
git checkout main
echo "test" >> test.txt
git add test.txt
git commit -m "test"
git push origin main
# → Error: protected branch ✅
```

---

## 📝 Siguiente Paso

Leer la documentación completa en:
- `docs/AGENTS.md`
- `docs/workflow/01-team-workflow.md`

---

**¿Problemas?** Ver troubleshooting en `docs/workflow/03-github-projects-setup.md`
