import { Grid } from './Grid';
import { GridRenderer } from './GridRenderer';
import { GraphVisualization } from './GraphVisualization';
import { BFSVisualization } from './BFSVisualization';
import { DFSVisualization } from './DFSVisualization';
import { EditMode, CellType, GridConfig } from './types';

// --- Interfaces ---

interface AgentState {
    // Grid configuration
    rows: number;
    cols: number;
    cellSize: number;

    // Edit mode
    currentMode: EditMode;

    // Grid and visualizations (internos, no reactivos)
    grid: Grid | null;
    renderer: GridRenderer | null;
    graphVisualization: GraphVisualization | null;
    bfsVisualization: BFSVisualization | null;
    dfsVisualization: DFSVisualization | null;

    // Estado del tablero
    isReady: boolean;
    startPosition: { row: number; col: number } | null;
    goalCount: number;

    // Datos de formulación matemática (reactivos)
    mathFormulation: {
        config: { rows: number; cols: number };
        startPos: { row: number; col: number } | null;
        goalPositions: { row: number; col: number }[];
        obstacles: { row: number; col: number }[];
        showTransitions: boolean;
    };

    // Estadísticas del grafo (reactivas)
    graphStats: {
        nodes: number;
        edges: number;
        avgDegree: string;
    };

    // Métodos
    init(): void;
    handleDimensionChange(): void;
    setMode(mode: EditMode): void;
    handleCellClick(row: number, col: number): void;
    handleClear(): void;
    handleClearObstacles(): void;
    handleGenerateGraph(): void;
    updateMathFormulation(): void;
    updateGraphVisualizations(): void;
    updateViews(): void;
    checkReadyState(): void;
    resetGraphPositions(): void;
    toggleGraphLabels(): void;
    zoomGraphIn(): void;
    zoomGraphOut(): void;
    resetGraphZoom(): void;
    getObstacles(): { row: number; col: number }[];
    toggleTransitions(): void;
}

// --- Lógica del Estado ---

function agentState(): AgentState {
    return {
        // Configuración inicial
        rows: 5,
        cols: 5,
        cellSize: 40,

        // Modo de edición
        currentMode: EditMode.SET_OBSTACLE,

        // Instancias (se inicializan en init())
        grid: null,
        renderer: null,
        graphVisualization: null,
        bfsVisualization: null,
        dfsVisualization: null,

        // Estado del tablero
        isReady: false,
        startPosition: null,
        goalCount: 0,

        // Datos de formulación matemática
        mathFormulation: {
            config: { rows: 5, cols: 5 },
            startPos: null,
            goalPositions: [],
            obstacles: [],
            showTransitions: false
        },

        // Estadísticas del grafo
        graphStats: {
            nodes: 0,
            edges: 0,
            avgDegree: '0'
        },

        // --- Métodos ---

        init() {
            console.log('[Alpine] Initializing agentState...');

            // Crear la configuración inicial
            const config: GridConfig = {
                rows: this.rows,
                cols: this.cols,
                cellSize: this.cellSize
            };

            // Inicializar el grid
            this.grid = new Grid(config);

            // Inicializar el renderizador
            this.renderer = new GridRenderer('grid-container', this.grid);

            // Configurar el handler de clicks en celdas
            this.renderer.setCellClickHandler((event: { row: number; col: number }) => {
                this.handleCellClick(event.row, event.col);
            });

            // Inicializar visualizaciones
            this.graphVisualization = new GraphVisualization('graph-visualization', this.grid);
            this.bfsVisualization = new BFSVisualization('bfs-visualization', this.grid);
            this.dfsVisualization = new DFSVisualization('dfs-visualization', this.grid);

            // Cargar ejemplo inicial
            this.loadInitialExample();

            // Renderizar todo
            this.renderer.render();
            this.updateViews();
            this.checkReadyState();

            console.log('[Alpine] Agent visualizer initialized successfully');
        },

        loadInitialExample() {
            if (!this.grid) return;

            // Estado inicio: (1,1) -> índice (0,0)
            this.grid.setStart(0, 0);

            // Estado final: (5,5) -> índice (4,4)
            this.grid.addGoal(4, 4);

            // Obstáculos del ejemplo
            this.grid.setObstacle(0, 2);
            this.grid.setObstacle(1, 2);
            this.grid.setObstacle(2, 2);
            this.grid.setObstacle(2, 1);
            this.grid.setObstacle(2, 3);
            this.grid.setObstacle(4, 3);

            console.log('[Alpine] Initial example loaded');
        },

        handleDimensionChange() {
            if (!this.grid || !this.renderer) return;

            // Validar dimensiones
            if (this.rows < 5 || this.rows > 30 || this.cols < 5 || this.cols > 30) {
                alert('Las dimensiones deben estar entre 5 y 30');
                return;
            }

            // Redimensionar el grid
            this.grid.resize(this.rows, this.cols);

            // Re-renderizar todo
            this.renderer.render();
            this.updateViews();
            this.checkReadyState();

            console.log(`[Alpine] Grid resized to ${this.rows}x${this.cols}`);
        },

        setMode(mode: EditMode) {
            this.currentMode = mode;
            console.log(`[Alpine] Mode changed to: ${mode}`);
        },

        handleCellClick(row: number, col: number) {
            if (!this.grid || !this.renderer) return;

            let updated = false;

            switch (this.currentMode) {
                case EditMode.SET_START:
                    updated = this.grid.setStart(row, col);
                    if (updated) {
                        console.log(`Start position set at (${row + 1}, ${col + 1})`);
                    }
                    break;

                case EditMode.SET_GOAL:
                    if (!this.grid.addGoal(row, col)) {
                        // Si ya es un destino, intentar eliminarlo
                        updated = this.grid.removeGoal(row, col);
                        if (updated) {
                            console.log(`Goal removed at (${row + 1}, ${col + 1})`);
                        }
                    } else {
                        updated = true;
                        console.log(`Goal added at (${row + 1}, ${col + 1})`);
                    }
                    break;

                case EditMode.SET_OBSTACLE:
                    // Si ya hay un obstáculo, eliminarlo (toggle)
                    const cellForObstacle = this.grid.getCell(row, col);
                    if (cellForObstacle && cellForObstacle.type === CellType.OBSTACLE) {
                        updated = this.grid.removeObstacle(row, col);
                        if (updated) {
                            console.log(`Obstacle removed at (${row + 1}, ${col + 1})`);
                        }
                    } else {
                        updated = this.grid.setObstacle(row, col);
                        if (updated) {
                            console.log(`Obstacle added at (${row + 1}, ${col + 1})`);
                        }
                    }
                    break;
            }

            if (updated) {
                this.renderer.update();
                this.updateMathFormulation();
                this.checkReadyState();
            }
        },

        handleClear() {
            if (!this.grid || !this.renderer) return;

            if (confirm('¿Estás seguro de que quieres limpiar todo el tablero?')) {
                this.grid.clear();
                this.renderer.update();
                this.updateMathFormulation();
                this.checkReadyState();
                console.log('[Alpine] Grid cleared');
            }
        },

        handleClearObstacles() {
            if (!this.grid || !this.renderer) return;

            this.grid.clearObstacles();
            this.renderer.update();
            this.updateMathFormulation();
            this.checkReadyState();
            console.log('[Alpine] Obstacles cleared');
        },

        handleGenerateGraph() {
            if (!this.grid) return;

            // Validar que el grid esté listo
            if (!this.grid.isReady()) {
                alert('Debes configurar al menos un estado inicial y un estado objetivo antes de generar el grafo.');
                return;
            }

            console.log('[Alpine] Generating graph visualizations...');

            // Cambiar a la pestaña del grafo (pestaña 1)
            window.dispatchEvent(new CustomEvent('change-tab', { detail: 1 }));

            // Esperar a que el DOM se actualice antes de renderizar
            setTimeout(() => {
                this.updateGraphVisualizations();
                console.log('[Alpine] Graph visualizations generated successfully');
            }, 300); // Esperar el tiempo de la transición de las pestañas
        },

        updateMathFormulation() {
            // Solo actualizar datos de formulación matemática (reactivo con Alpine)
            if (this.grid) {
                this.mathFormulation.config = this.grid.getConfig();
                this.mathFormulation.startPos = this.grid.getStartPosition();
                this.mathFormulation.goalPositions = this.grid.getGoalPositions();
                this.mathFormulation.obstacles = this.getObstacles();
            }
        },

        updateGraphVisualizations() {
            // Actualizar visualizaciones D3 (grafo, BFS, DFS)
            if (this.graphVisualization) {
                const stats = this.graphVisualization.update();
                this.graphStats.nodes = stats.nodes;
                this.graphStats.edges = stats.edges;
                this.graphStats.avgDegree = stats.avgDegree;
            }
            if (this.bfsVisualization) {
                this.bfsVisualization.update();
            }
            if (this.dfsVisualization) {
                this.dfsVisualization.update();
            }
        },

        updateViews() {
            // Método legacy que actualiza todo (usado solo en init y dimensionChange)
            this.updateMathFormulation();
            this.updateGraphVisualizations();
        },

        // Métodos de control del grafo
        resetGraphPositions() {
            if (this.graphVisualization && this.graphStats.nodes > 0) {
                this.graphVisualization.resetPositions();
            }
        },

        toggleGraphLabels() {
            if (this.graphVisualization && this.graphStats.nodes > 0) {
                this.graphVisualization.toggleLabels();
            }
        },

        zoomGraphIn() {
            if (this.graphVisualization && this.graphStats.nodes > 0) {
                this.graphVisualization.zoomIn();
            }
        },

        zoomGraphOut() {
            if (this.graphVisualization && this.graphStats.nodes > 0) {
                this.graphVisualization.zoomOut();
            }
        },

        resetGraphZoom() {
            if (this.graphVisualization && this.graphStats.nodes > 0) {
                this.graphVisualization.resetZoom();
            }
        },

        getObstacles() {
            const obstacles: { row: number; col: number }[] = [];
            if (!this.grid) return obstacles;

            const cells = this.grid.getAllCells();
            for (let row = 0; row < cells.length; row++) {
                for (let col = 0; col < cells[row].length; col++) {
                    if (cells[row][col].type === 'obstacle') {
                        obstacles.push({ row, col });
                    }
                }
            }
            return obstacles;
        },

        toggleTransitions() {
            this.mathFormulation.showTransitions = !this.mathFormulation.showTransitions;
        },

        checkReadyState() {
            if (!this.grid) return;

            this.isReady = this.grid.isReady();
            this.startPosition = this.grid.getStartPosition();
            this.goalCount = this.grid.getGoalPositions().length;

            console.log(`[Alpine] Grid ready: ${this.isReady}`);
        }
    } as unknown as AgentState;
}

// Extender la interfaz global de Window
declare global {
    interface Window {
        agentState: () => AgentState;
    }
}

window.agentState = agentState;
