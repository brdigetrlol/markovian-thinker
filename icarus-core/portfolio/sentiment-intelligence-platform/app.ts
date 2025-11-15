/**
 * Sentiment Intelligence Platform - Main Application (TypeScript)
 * Real-time 3D visualization and analytics dashboard
 */

import * as THREE from 'three';
import { Chart, ChartConfiguration, ChartData } from 'chart.js';

// =============================================================================
// TYPE DEFINITIONS
// =============================================================================

type SentimentClass = 'positive' | 'negative' | 'neutral';
type DataSource = 'twitter' | 'reviews' | 'support' | 'news' | 'api' | 'all';

interface SentimentData {
    text: string;
    sentiment: SentimentClass;
    score: number;
    source: DataSource;
    timestamp: Date;
    confidence: number;
}

interface APIResponse {
    sentiment: SentimentClass;
    compound_score: number;
    positive_score: number;
    negative_score: number;
    neutral_score: number;
    confidence: number;
    category?: string;
}

interface Configuration {
    API_URL: string;
    UPDATE_INTERVAL: number;
    MAX_FEED_ITEMS: number;
    MAX_3D_POINTS: number;
    USE_REAL_API: boolean;
}

interface ApplicationState {
    isRunning: boolean;
    currentSource: DataSource;
    totalAnalyzed: number;
    sentimentHistory: SentimentData[];
    feedPaused: boolean;
    rotation: { x: number; y: number };
}

interface SampleTexts {
    positive: string[];
    negative: string[];
    neutral: string[];
}

// =============================================================================
// CONFIGURATION & STATE
// =============================================================================

const CONFIG: Configuration = {
    API_URL: 'http://localhost:3000/api/v1',
    UPDATE_INTERVAL: 2000,
    MAX_FEED_ITEMS: 50,
    MAX_3D_POINTS: 200,
    USE_REAL_API: false
};

const STATE: ApplicationState = {
    isRunning: true,
    currentSource: 'all',
    totalAnalyzed: 0,
    sentimentHistory: [],
    feedPaused: false,
    rotation: { x: 0, y: 0 }
};

// =============================================================================
// THREE.JS 3D VISUALIZATION
// =============================================================================

class SentimentVisualization {
    private scene: THREE.Scene;
    private camera: THREE.PerspectiveCamera;
    private renderer: THREE.WebGLRenderer;
    private sentimentGroup: THREE.Group;
    private raycaster: THREE.Raycaster;
    private mouse: THREE.Vector2;
    private container: HTMLElement;

    constructor(containerId: string) {
        this.container = document.getElementById(containerId)!;
        if (!this.container) {
            throw new Error(`Container #${containerId} not found`);
        }

        const width = this.container.clientWidth;
        const height = this.container.clientHeight;

        // Scene setup
        this.scene = new THREE.Scene();
        this.scene.fog = new THREE.Fog(0x0f172a, 10, 50);

        // Camera setup
        this.camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
        this.camera.position.z = 15;
        this.camera.position.y = 5;
        this.camera.lookAt(0, 0, 0);

        // Renderer setup
        this.renderer = new THREE.WebGLRenderer({
            antialias: true,
            alpha: true
        });
        this.renderer.setSize(width, height);
        this.renderer.setPixelRatio(window.devicePixelRatio);
        this.container.appendChild(this.renderer.domElement);

        // Sentiment data group
        this.sentimentGroup = new THREE.Group();
        this.scene.add(this.sentimentGroup);

        // Lighting
        this.setupLighting();

        // Grid helper
        const gridHelper = new THREE.GridHelper(30, 30, 0x334155, 0x1e293b);
        gridHelper.position.y = -5;
        this.scene.add(gridHelper);

        // Raycaster for interactivity
        this.raycaster = new THREE.Raycaster();
        this.mouse = new THREE.Vector2();

        // Event listeners
        this.setupEventListeners();

        // Start animation
        this.animate();
    }

    private setupLighting(): void {
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
        this.scene.add(ambientLight);

        const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight.position.set(10, 10, 10);
        this.scene.add(directionalLight);
    }

    private setupEventListeners(): void {
        window.addEventListener('resize', () => this.onWindowResize());
        this.container.addEventListener('mousemove', (e) => this.onMouseMove(e));
        this.container.addEventListener('click', (e) => this.onMouseClick(e));
    }

    private createSentimentSphere(
        sentiment: SentimentClass,
        position: THREE.Vector3,
        size: number = 0.3
    ): THREE.Mesh {
        const colorMap: Record<SentimentClass, number> = {
            positive: 0x10b981,
            negative: 0xef4444,
            neutral: 0x64748b
        };

        const color = colorMap[sentiment];

        const geometry = new THREE.SphereGeometry(size, 16, 16);
        const material = new THREE.MeshPhongMaterial({
            color,
            emissive: color,
            emissiveIntensity: 0.3,
            shininess: 100,
            transparent: true,
            opacity: 0.9
        });

        const sphere = new THREE.Mesh(geometry, material);
        sphere.position.copy(position);
        sphere.userData = { sentiment, timestamp: Date.now() };

        // Add glow effect
        const glowGeometry = new THREE.SphereGeometry(size * 1.5, 16, 16);
        const glowMaterial = new THREE.MeshBasicMaterial({
            color,
            transparent: true,
            opacity: 0.2
        });
        const glow = new THREE.Mesh(glowGeometry, glowMaterial);
        sphere.add(glow);

        return sphere;
    }

    public addSentimentPoint(sentimentData: SentimentData): void {
        const x = (Math.random() - 0.5) * 20;
        const y = (Math.random() - 0.5) * 10;
        const z = this.sentimentGroup.children.length * -0.2;

        const position = new THREE.Vector3(x, y, z);
        const sphere = this.createSentimentSphere(sentimentData.sentiment, position);

        this.sentimentGroup.add(sphere);

        // Remove old points
        if (this.sentimentGroup.children.length > CONFIG.MAX_3D_POINTS) {
            const oldSphere = this.sentimentGroup.children[0];
            this.sentimentGroup.remove(oldSphere);
            if (oldSphere instanceof THREE.Mesh) {
                oldSphere.geometry.dispose();
                if (oldSphere.material instanceof THREE.Material) {
                    oldSphere.material.dispose();
                }
            }
        }

        // Follow timeline
        if (this.sentimentGroup.children.length > 10) {
            this.camera.position.z = 15 + this.sentimentGroup.children.length * 0.05;
        }
    }

    private animate = (): void => {
        requestAnimationFrame(this.animate);

        // Rotate group
        this.sentimentGroup.rotation.y += 0.002;
        STATE.rotation.y += 0.002;

        // Animate spheres
        this.sentimentGroup.children.forEach((obj, index) => {
            if (obj instanceof THREE.Mesh) {
                obj.position.y += Math.sin(Date.now() * 0.001 + index) * 0.002;
                obj.rotation.y += 0.01;
            }
        });

        this.renderer.render(this.scene, this.camera);
    };

    private onWindowResize(): void {
        const width = this.container.clientWidth;
        const height = this.container.clientHeight;

        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(width, height);
    }

    private onMouseMove(event: MouseEvent): void {
        const rect = this.container.getBoundingClientRect();
        this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    }

    private onMouseClick(event: MouseEvent): void {
        this.raycaster.setFromCamera(this.mouse, this.camera);
        const intersects = this.raycaster.intersectObjects(this.sentimentGroup.children);

        if (intersects.length > 0) {
            const sphere = intersects[0].object;
            if (sphere instanceof THREE.Mesh && sphere.material instanceof THREE.MeshPhongMaterial) {
                sphere.material.emissiveIntensity = 1.0;
                setTimeout(() => {
                    sphere.material.emissiveIntensity = 0.3;
                }, 500);
            }
        }
    }

    public rotateView(): void {
        const targetRotation = this.sentimentGroup.rotation.y + Math.PI / 2;
        const duration = 1000;
        const start = Date.now();
        const startRotation = this.sentimentGroup.rotation.y;

        const animateRotation = (): void => {
            const elapsed = Date.now() - start;
            const progress = Math.min(elapsed / duration, 1);
            const easeProgress = 1 - Math.pow(1 - progress, 3);

            this.sentimentGroup.rotation.y = startRotation + (targetRotation - startRotation) * easeProgress;

            if (progress < 1) {
                requestAnimationFrame(animateRotation);
            }
        };

        animateRotation();
    }

    public resetView(): void {
        this.camera.position.set(0, 5, 15);
        this.camera.lookAt(0, 0, 0);
        this.sentimentGroup.rotation.set(0, 0, 0);
        STATE.rotation = { x: 0, y: 0 };
    }
}

// =============================================================================
// CHART.JS VISUALIZATIONS
// =============================================================================

class DashboardCharts {
    private trendChart: Chart;
    private sourceChart: Chart;

    constructor() {
        this.trendChart = this.initTrendChart();
        this.sourceChart = this.initSourceChart();
    }

    private initTrendChart(): Chart {
        const ctx = (document.getElementById('trendChart') as HTMLCanvasElement).getContext('2d')!;

        const config: ChartConfiguration = {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Positive',
                        data: [],
                        borderColor: '#10b981',
                        backgroundColor: 'rgba(16, 185, 129, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Negative',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Neutral',
                        data: [],
                        borderColor: '#64748b',
                        backgroundColor: 'rgba(100, 116, 139, 0.1)',
                        tension: 0.4,
                        fill: true
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: true,
                        position: 'top',
                        labels: {
                            color: '#e2e8f0',
                            font: { size: 12 }
                        }
                    }
                },
                scales: {
                    x: {
                        grid: { color: '#334155' },
                        ticks: { color: '#94a3b8' }
                    },
                    y: {
                        grid: { color: '#334155' },
                        ticks: { color: '#94a3b8' }
                    }
                }
            }
        };

        return new Chart(ctx, config);
    }

    private initSourceChart(): Chart {
        const ctx = (document.getElementById('sourceChart') as HTMLCanvasElement).getContext('2d')!;

        const config: ChartConfiguration = {
            type: 'doughnut',
            data: {
                labels: ['Twitter/X', 'Reviews', 'Support', 'News'],
                datasets: [{
                    data: [1200, 856, 234, 445],
                    backgroundColor: ['#3b82f6', '#10b981', '#f59e0b', '#8b5cf6'],
                    borderColor: '#1e293b',
                    borderWidth: 3
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'right',
                        labels: {
                            color: '#e2e8f0',
                            font: { size: 12 },
                            padding: 15
                        }
                    }
                }
            }
        };

        return new Chart(ctx, config);
    }

    public updateTrendChart(timestamp: string, positive: number, negative: number, neutral: number): void {
        const maxPoints = 20;

        this.trendChart.data.labels!.push(timestamp);
        this.trendChart.data.datasets[0].data.push(positive);
        this.trendChart.data.datasets[1].data.push(negative);
        this.trendChart.data.datasets[2].data.push(neutral);

        if (this.trendChart.data.labels!.length > maxPoints) {
            this.trendChart.data.labels!.shift();
            this.trendChart.data.datasets.forEach(dataset => dataset.data.shift());
        }

        this.trendChart.update('none');
    }
}

// =============================================================================
// DATA GENERATION & SIMULATION
// =============================================================================

class DataGenerator {
    private readonly sampleTexts: SampleTexts = {
        positive: [
            "This product is absolutely amazing! Best purchase ever!",
            "Excellent customer service, highly recommend!",
            "Love the new features, works perfectly!",
            "Outstanding quality, exceeded my expectations!",
            "Great experience from start to finish!",
            "Fantastic support team, very helpful!",
            "This is exactly what I needed, perfect!",
            "Incredible value for money, so happy!"
        ],
        negative: [
            "Very disappointed with the quality",
            "Terrible customer service, no response",
            "Product stopped working after one day",
            "Would not recommend, waste of money",
            "Frustrating experience, many bugs",
            "Poor quality, returning immediately",
            "Awful support, still waiting for help",
            "Complete disaster, avoid at all costs"
        ],
        neutral: [
            "The product arrived on time",
            "It works as described in manual",
            "Standard features, nothing special",
            "Delivered as expected",
            "Average quality for the price",
            "Typical support response time",
            "Meets basic requirements",
            "No issues so far, just okay"
        ]
    };

    private readonly sources: DataSource[] = ['twitter', 'reviews', 'support', 'news'];

    public generate(): SentimentData {
        const rand = Math.random();
        let sentiment: SentimentClass;
        let texts: string[];

        if (rand < 0.5) {
            sentiment = 'positive';
            texts = this.sampleTexts.positive;
        } else if (rand < 0.8) {
            sentiment = 'neutral';
            texts = this.sampleTexts.neutral;
        } else {
            sentiment = 'negative';
            texts = this.sampleTexts.negative;
        }

        const text = texts[Math.floor(Math.random() * texts.length)];
        const source = this.sources[Math.floor(Math.random() * this.sources.length)];
        const score = sentiment === 'positive' ? 0.6 + Math.random() * 0.4 :
                      sentiment === 'negative' ? -0.6 - Math.random() * 0.4 :
                      (Math.random() - 0.5) * 0.4;

        return {
            text,
            sentiment,
            score,
            source,
            timestamp: new Date(),
            confidence: 0.85 + Math.random() * 0.14
        };
    }

    public async analyzeWithAPI(text: string): Promise<SentimentData> {
        if (!CONFIG.USE_REAL_API) {
            return this.generate();
        }

        try {
            const response = await fetch(`${CONFIG.API_URL}/analyze`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text })
            });

            if (!response.ok) {
                throw new Error('API request failed');
            }

            const data: APIResponse = await response.json();

            return {
                text,
                sentiment: data.sentiment,
                score: data.compound_score,
                source: 'api',
                timestamp: new Date(),
                confidence: data.confidence
            };
        } catch (error) {
            console.warn('API unavailable, using simulation:', error);
            CONFIG.USE_REAL_API = false;
            return this.generate();
        }
    }
}

// =============================================================================
// REAL-TIME FEED
// =============================================================================

class SentimentFeed {
    private container: HTMLElement;
    private readonly sourceIcons: Record<string, string> = {
        twitter: '🐦',
        reviews: '⭐',
        support: '💬',
        news: '📰',
        api: '🤖'
    };

    constructor(containerId: string) {
        this.container = document.getElementById(containerId)!;
        if (!this.container) {
            throw new Error(`Feed container #${containerId} not found`);
        }
    }

    public addItem(data: SentimentData): void {
        if (STATE.feedPaused) return;

        const feedItem = document.createElement('div');
        feedItem.className = `feed-item ${data.sentiment}`;

        const time = data.timestamp.toLocaleTimeString();
        const sourceIcon = this.sourceIcons[data.source] || '📊';
        const sourceName = data.source.charAt(0).toUpperCase() + data.source.slice(1);

        feedItem.innerHTML = `
            <div class="feed-header">
                <div class="feed-source">
                    <span>${sourceIcon}</span>
                    <span>${sourceName}</span>
                </div>
                <div class="feed-time">${time}</div>
            </div>
            <div class="feed-text">${this.escapeHtml(data.text)}</div>
            <div>
                <span class="feed-sentiment ${data.sentiment}">
                    ${data.sentiment.toUpperCase()} (${data.score.toFixed(2)})
                </span>
            </div>
        `;

        this.container.insertBefore(feedItem, this.container.firstChild);

        // Remove old items
        while (this.container.children.length > CONFIG.MAX_FEED_ITEMS) {
            this.container.removeChild(this.container.lastChild);
        }
    }

    public clear(): void {
        this.container.innerHTML = '';
    }

    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// =============================================================================
// METRICS & STATISTICS
// =============================================================================

class MetricsCalculator {
    public static update(charts: DashboardCharts): void {
        const recentData = STATE.sentimentHistory.slice(-100);
        if (recentData.length === 0) return;

        const positive = recentData.filter(d => d.sentiment === 'positive').length;
        const negative = recentData.filter(d => d.sentiment === 'negative').length;
        const neutral = recentData.filter(d => d.sentiment === 'neutral').length;

        const avgScore = recentData.reduce((sum, d) => sum + d.score, 0) / recentData.length;
        const nps = Math.round(((positive - negative) / recentData.length) * 100);

        // Update header stats
        this.updateElement('totalAnalyzed', STATE.totalAnalyzed.toLocaleString());
        this.updateElement('avgSentiment', avgScore.toFixed(2));
        this.updateElement('npsScore', nps > 0 ? `+${nps}` : nps.toString());

        // Update chart
        const now = new Date().toLocaleTimeString();
        charts.updateTrendChart(now, positive, negative, neutral);
    }

    private static updateElement(id: string, value: string): void {
        const element = document.getElementById(id);
        if (element) {
            element.textContent = value;
        }
    }
}

// =============================================================================
// USER CONTROLS
// =============================================================================

class UserControls {
    constructor(
        private visualization: SentimentVisualization,
        private feed: SentimentFeed
    ) {}

    public rotateView(): void {
        this.visualization.rotateView();
    }

    public resetView(): void {
        this.visualization.resetView();
    }

    public pauseFeed(button: HTMLButtonElement): void {
        STATE.feedPaused = !STATE.feedPaused;
        button.textContent = STATE.feedPaused ? '▶ Resume' : '⏸ Pause';
    }

    public clearFeed(): void {
        this.feed.clear();
    }

    public generateReport(): void {
        alert('📊 Report Generation\n\nGenerating comprehensive sentiment analysis report...\n\n' +
              'This would include:\n' +
              '• Executive summary\n' +
              '• Trend analysis\n' +
              '• Key insights\n' +
              '• Actionable recommendations\n' +
              '• Exportable to PDF/Excel\n\n' +
              'In production, this connects to the reporting engine.');
    }

    public exportData(): void {
        const csvData = STATE.sentimentHistory.slice(-100).map(d =>
            `${d.timestamp.toISOString()},${d.source},${d.sentiment},${d.score},${d.text.replace(/,/g, ';')}`
        ).join('\n');

        const header = 'Timestamp,Source,Sentiment,Score,Text\n';
        const csv = header + csvData;

        const blob = new Blob([csv], { type: 'text/csv' });
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `sentiment_export_${Date.now()}.csv`;
        a.click();
        window.URL.revokeObjectURL(url);

        console.log('Exported', STATE.sentimentHistory.length, 'records');
    }

    public configureAlerts(): void {
        alert('🔔 Alert Configuration\n\n' +
              'Configure intelligent alerts:\n\n' +
              '• Negative sentiment spikes\n' +
              '• Volume thresholds\n' +
              '• Specific keywords\n' +
              '• Source-based triggers\n' +
              '• Custom conditions\n\n' +
              'Delivery methods:\n' +
              '• Email notifications\n' +
              '• Slack integration\n' +
              '• Webhooks\n' +
              '• SMS alerts\n\n' +
              'In production, this opens the alert configuration panel.');
    }
}

// =============================================================================
// MAIN APPLICATION
// =============================================================================

class SentimentPlatform {
    private visualization: SentimentVisualization;
    private charts: DashboardCharts;
    private feed: SentimentFeed;
    private dataGenerator: DataGenerator;
    private controls: UserControls;
    private updateInterval: number | null = null;

    constructor() {
        console.log('🧠 Initializing Sentiment Intelligence Platform...');

        this.visualization = new SentimentVisualization('threejs-container');
        this.charts = new DashboardCharts();
        this.feed = new SentimentFeed('feedContainer');
        this.dataGenerator = new DataGenerator();
        this.controls = new UserControls(this.visualization, this.feed);

        this.setupDataSourceFiltering();
        this.exposeGlobalControls();
        this.startDataStream();
        this.checkAPIStatus();

        console.log('✅ Platform initialized successfully!');
        console.log('📊 Real-time monitoring active');
        console.log('🎨 3D visualization running');
    }

    private setupDataSourceFiltering(): void {
        const sources = document.querySelectorAll('.data-source');

        sources.forEach(source => {
            source.addEventListener('click', function(this: HTMLElement) {
                sources.forEach(s => s.classList.remove('active'));
                this.classList.add('active');

                const dataSource = this.getAttribute('data-source') as DataSource;
                STATE.currentSource = dataSource;

                console.log('Filtering by source:', STATE.currentSource);
            });
        });
    }

    private exposeGlobalControls(): void {
        (window as any).rotateView = () => this.controls.rotateView();
        (window as any).resetView = () => this.controls.resetView();
        (window as any).pauseFeed = (btn: HTMLButtonElement) => this.controls.pauseFeed(btn);
        (window as any).clearFeed = () => this.controls.clearFeed();
        (window as any).generateReport = () => this.controls.generateReport();
        (window as any).exportData = () => this.controls.exportData();
        (window as any).configureAlerts = () => this.controls.configureAlerts();
    }

    private startDataStream(): void {
        // Initial data burst
        for (let i = 0; i < 10; i++) {
            setTimeout(() => this.updateDashboard(), i * 200);
        }

        // Continuous updates
        this.updateInterval = window.setInterval(() => this.updateDashboard(), CONFIG.UPDATE_INTERVAL);
    }

    private async updateDashboard(): Promise<void> {
        if (!STATE.isRunning) return;

        try {
            const data = await this.dataGenerator.analyzeWithAPI('Sample text for analysis');

            STATE.totalAnalyzed++;
            STATE.sentimentHistory.push(data);

            this.visualization.addSentimentPoint(data);
            this.feed.addItem(data);
            MetricsCalculator.update(this.charts);

            // Keep history manageable
            if (STATE.sentimentHistory.length > 1000) {
                STATE.sentimentHistory = STATE.sentimentHistory.slice(-500);
            }
        } catch (error) {
            console.error('Update error:', error);
        }
    }

    private async checkAPIStatus(): Promise<void> {
        try {
            const response = await fetch(`${CONFIG.API_URL}/health`);
            if (response.ok) {
                CONFIG.USE_REAL_API = true;
                console.log('✅ Connected to Rust API backend');
            }
        } catch (error) {
            console.log('ℹ️ Running in demo mode (API not connected)');
            console.log('   Start the Rust API to enable live analysis');
        }
    }

    public stop(): void {
        STATE.isRunning = false;
        if (this.updateInterval !== null) {
            clearInterval(this.updateInterval);
        }
    }
}

// =============================================================================
// START APPLICATION
// =============================================================================

let platform: SentimentPlatform;

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        platform = new SentimentPlatform();
    });
} else {
    platform = new SentimentPlatform();
}

export { SentimentPlatform, SentimentData, DataSource, SentimentClass };
