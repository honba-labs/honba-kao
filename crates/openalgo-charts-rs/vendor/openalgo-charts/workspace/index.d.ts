import { DataVariant, ChartState, ChartSettingsState, LinkMissingPolicy, IndicatorState, PaneState, PriceScaleId, ChartRestoreOptions, Chart, InstrumentKey } from 'openalgo-charts';

declare class WorkspaceDocumentError extends Error {
    constructor(message: string);
}

declare const WORKSPACE_VERSION = 1;
type WorkspaceKind = 'workspace' | 'indicator-template';
type WorkspaceSettings = Record<string, string | number | boolean>;
type WorkspaceChartState = ChartState & ChartSettingsState & {
    timezone?: string;
};
interface WorkspaceComparison {
    id: string;
    symbol: string;
    exchange: string;
    color?: string;
    visible: boolean;
}
interface WorkspaceSlot {
    paneId: string;
    row: number;
    column: number;
    rowSpan: number;
    columnSpan: number;
}
interface WorkspacePane {
    id: string;
    symbol: string;
    exchange: string;
    interval: string;
    chartType: string;
    /** Which of the provider's series the chart showed. Absent is the provider's default. */
    variant?: DataVariant;
    chart: WorkspaceChartState;
    settings: WorkspaceSettings;
    volume: boolean;
    magnet: 'off' | 'weak' | 'strong';
    stay: boolean;
    comparisons: WorkspaceComparison[];
    comparisonMode: 'price' | 'percent';
    historyPeriod?: string | undefined;
    /**
     * The id of the named link group this chart is in, one of `sync.groups`.
     * Absent: in no group when the desk declares groups, and in the desk's one
     * group when it declares none, which is how every earlier workspace reads.
     */
    linkGroup?: string;
}
/**
 * The channels one link group links on, named as `LinkOptions` names them, so
 * a group's flags open as a `LinkGroup` without translation. The first four
 * have been written since the first release and stay required; the rest are
 * optional, and absent reads as the engine's default (off, and 'nearest').
 */
interface WorkspaceLinkChannels {
    crosshair: boolean;
    viewport: boolean;
    symbol: boolean;
    interval: boolean;
    appearance?: boolean;
    /** The chart type (candles, bars, a line) follows. */
    chartType?: boolean;
    /** Drawings are shared between the group's charts on the same instrument. */
    drawings?: boolean;
    /** What a follower does with an instant it has no bar for. */
    whenMissing?: LinkMissingPolicy;
}
/** A named link group: the charts whose `linkGroup` is its id link on its own channels. */
interface WorkspaceLinkGroup extends WorkspaceLinkChannels {
    /** What each chart's `linkGroup` names: 1 to 100 characters, unique in the workspace. */
    id: string;
    /** What a person called the group: 1 to 120 characters. */
    name: string;
}
/** How the charts of a workspace link to each other. */
interface WorkspaceSync extends WorkspaceLinkChannels {
    /**
     * Named link groups, up to 16. Absent: the whole desk is one group on the
     * channels above, as every workspace saved before groups is. Present, even
     * empty: each chart is in the group its `linkGroup` names or in none, and
     * each group links on its own channels. The channels above then only tell a
     * reader that predates groups what to apply to the whole desk, so a writer
     * turns one on there only when every chart can follow it together.
     */
    groups?: WorkspaceLinkGroup[];
}
interface WorkspacePayload {
    layout: {
        rows: number;
        columns: number;
        slots: WorkspaceSlot[];
        preset?: string;
        rowWeights?: number[];
        columnWeights?: number[];
    };
    panes: WorkspacePane[];
    activePaneId: string;
    sync: WorkspaceSync;
}
interface DocumentMetadata {
    version: 1;
    id: string;
    name: string;
    createdAt: number;
    updatedAt: number;
}
interface WorkspaceDocument extends DocumentMetadata, WorkspacePayload {
    kind: 'workspace';
}
interface IndicatorTemplatePlotBinding {
    instanceId: string;
    plotKey: string;
    paneIndex: number;
    scaleId: PriceScaleId;
}
/**
 * Pane and scale relationships of a portable template. Pane 0 is always the
 * price pane and the study panes follow in their chart order, whatever slot
 * the price pane held on the chart that was captured: a template applies the
 * same way to a chart that keeps its price pane at the top and to one that
 * keeps it below its studies.
 */
interface IndicatorTemplateLayout {
    panes: PaneState[];
    plots: IndicatorTemplatePlotBinding[];
    primaryScaleId?: PriceScaleId;
}
interface IndicatorTemplatePayload {
    indicators: IndicatorState[];
    layout?: IndicatorTemplateLayout;
}
type IndicatorTemplateInput = IndicatorState[] | IndicatorTemplatePayload;
interface IndicatorTemplateDocument extends DocumentMetadata, IndicatorTemplatePayload {
    kind: 'indicator-template';
}
/** Keep repeated/custom descriptor IDs; availability is checked by the applying host. */
declare function parseIndicatorStates(input: unknown): IndicatorState[];

/** Validate and detach portable study settings and optional pane/scale relationships. */
declare function parseIndicatorTemplatePayload(input: unknown): IndicatorTemplatePayload;
/** Validate, detach and project a portable chart-only workspace. */
declare function parseWorkspaceDocument(input: unknown): WorkspaceDocument;
declare function parseWorkspacePayload(input: unknown): WorkspacePayload;
declare function parseIndicatorTemplate(input: unknown): IndicatorTemplateDocument;
/** Explicit migration of the existing single-widget state; no input is modified. */
declare function migrateWidgetWorkspace(input: unknown, meta: {
    id: string;
    name: string;
    now: number;
}): WorkspaceDocument;

type IndicatorTemplateMode = 'replace' | 'append';
/**
 * Validate and detach a study plan before the host changes a chart. Append keeps
 * current instance identities and places positive incoming pane groups after the
 * existing panes. Incoming pane zero is the price pane, which on the chart sits
 * at `primaryPaneIndex` (0 unless the host moved it below its studies); current
 * studies are in the chart's own slots. Dependency copies receive fresh
 * identities; legacy independent studies leave identity allocation to the chart.
 * Pass the chart's pane count as nextPaneIndex. This function creates no chart or UI.
 */
declare function planIndicatorTemplate(current: IndicatorState[], incoming: IndicatorState[], mode: IndicatorTemplateMode, available: ReadonlySet<string>, nextPaneIndex: number, primaryPaneIndex?: number): IndicatorState[];

interface IndicatorTemplateApplyOptions {
    /** Copy isolates non-primary main-pane scales; share keeps existing destination settings. */
    scalePolicy?: 'copy' | 'share';
    /** Auto fits copied axes to current data or their fixed band; preserve retains saved views. */
    rangePolicy?: 'auto' | 'preserve';
}
interface IndicatorTemplatePlan {
    indicators: IndicatorState[];
    panes?: PaneState[];
    /**
     * Slot of the price pane in `panes`, present only when the destination keeps
     * its price pane below a study pane. Forward it beside `panes` as the
     * restored state's `primaryPane` (a version 2 state), or the restore puts
     * the price pane back at the top and lays its settings on a study pane.
     */
    primaryPane?: number;
    /** Forward as restoreState's second argument to retain destination runtime formatters. */
    restoreOptions?: ChartRestoreOptions;
}
/**
 * Capture effective bindings and scale configuration, without data or runtime
 * callbacks. The template is written price pane first (see
 * `IndicatorTemplateLayout`), so a chart whose price pane sits below its
 * studies captures the same portable document as one whose price pane is on top.
 */
declare function captureIndicatorTemplate(chart: Chart): IndicatorTemplatePayload;
/**
 * Plan a detached patch. The host owns restore, cancellation and recovery.
 *
 * The plan is in the destination's own slots, and keeps its price pane where
 * it is: a template's price pane is the destination's price pane, and its
 * study panes take the slots around it in order. When that slot is not 0 the
 * plan carries it as `primaryPane`, and lists the destination's panes.
 */
declare function planIndicatorTemplateState(chart: Chart, incoming: IndicatorTemplateInput, mode: IndicatorTemplateMode, options?: IndicatorTemplateApplyOptions): IndicatorTemplatePlan;

interface WorkspaceCatalog {
    version: 1;
    revision: number;
    workspaces: WorkspaceDocument[];
    templates: IndicatorTemplateDocument[];
    recentWorkspaceIds: string[];
    activeWorkspaceId: string | null;
    autosave: boolean;
}
interface WorkspaceOperationOptions {
    /** Cancellation is effective until the storage transaction commits. */
    signal?: AbortSignal | undefined;
    /**
     * Refuse the change with `WorkspaceConflictError` when the catalog has moved
     * past this revision. Pass the revision the change was prepared from, so a
     * layout another session saved in the meantime is never overwritten unseen.
     */
    expectedRevision?: number;
}
interface WorkspaceOpenOptions extends WorkspaceOperationOptions {
    /** Reject if a grid was prepared from a catalog that has since changed. */
    expectedRevision?: number;
}
/** The host must atomically reject writes whose expectedRevision is stale. */
interface WorkspaceStorage {
    read(namespace: string): Promise<unknown | null>;
    write(namespace: string, catalog: WorkspaceCatalog, expectedRevision: number, options?: WorkspaceOperationOptions): Promise<void>;
}
interface WorkspaceRepositoryOptions {
    now?: () => number;
    id?: () => string;
}
/**
 * What a layouts control calls, and nothing more. `WorkspaceRepository`
 * implements it, with `importDocument` and `exportDocument` besides; a host
 * that keeps layouts on its own server can implement it directly. The widget
 * tier takes this contract as a type, so a widget host loads the workspace
 * tier only when it passes one.
 *
 * A change given `expectedRevision` is refused with an error named
 * `WorkspaceConflictError` unless the catalog is at that revision, and a
 * change that commits moves the catalog forward by exactly one revision.
 */
interface WorkspaceStore {
    load(): Promise<WorkspaceCatalog>;
    /**
     * Called with a detached copy of the catalog after each change this store
     * commits, before that change's promise resolves: a control learns the
     * revision its next change is checked against here. Returns the unsubscribe.
     */
    subscribe(listener: (catalog: WorkspaceCatalog) => void): () => void;
    createWorkspace(name: string, input: WorkspacePayload, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument>;
    saveWorkspace(id: string, input: WorkspacePayload, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument>;
    /** Record the active layout and the recent list. It restores nothing: the caller applies the document. */
    openWorkspace(id: string, options?: WorkspaceOpenOptions): Promise<WorkspaceDocument>;
    createTemplate(name: string, input: IndicatorTemplateInput, options?: WorkspaceOperationOptions): Promise<IndicatorTemplateDocument>;
    saveTemplate(id: string, input: IndicatorTemplateInput, options?: WorkspaceOperationOptions): Promise<IndicatorTemplateDocument>;
    rename(kind: WorkspaceKind, id: string, name: string, options?: WorkspaceOperationOptions): Promise<void>;
    duplicate(kind: WorkspaceKind, id: string, name: string, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument | IndicatorTemplateDocument>;
    remove(kind: WorkspaceKind, id: string, options?: WorkspaceOperationOptions): Promise<void>;
    /** Store the autosave preference. It starts no timer: the control that holds a layout saves it. */
    setAutosave(enabled: boolean, options?: WorkspaceOperationOptions): Promise<void>;
    /**
     * `captureIndicatorTemplate` (since 2.5.10): a chart's studies with their
     * panes and scales, for `createTemplate`. The widget's indicator picker reads
     * it through the store it is given, so the widget tier never loads this tier
     * on its own. Without it the picker saves the plain study list.
     */
    captureIndicatorTemplate?(chart: Chart): IndicatorTemplatePayload;
    /**
     * `planIndicatorTemplateState` (since 2.5.10), for the same reason. The
     * picker offers saved templates only when the store has it: applying one
     * without a plan could lose the chart's panes and scales.
     */
    planIndicatorTemplateState?(chart: Chart, incoming: IndicatorTemplateInput, mode: IndicatorTemplateMode, options?: IndicatorTemplateApplyOptions): IndicatorTemplatePlan;
}
declare class WorkspaceConflictError extends Error {
    constructor();
}
/** Parse the complete catalog before any mutation; corrupt storage is never replaced. */
declare function parseWorkspaceCatalog(input: unknown): WorkspaceCatalog;
/**
 * Named configuration with serialized, revision-checked writes. Each change is
 * applied to the catalog as stored at that moment, so an edit made in another
 * session survives; a change given `expectedRevision`, or a write racing
 * another session, is refused as a conflict instead. Create a new repository
 * when the account changes; an in-flight operation keeps its owner.
 */
declare class WorkspaceRepository implements WorkspaceStore {
    private readonly _catalog;
    private readonly _now;
    private readonly _id;
    constructor(storage: WorkspaceStorage, namespace: string, options?: WorkspaceRepositoryOptions);
    get namespace(): string;
    load(): Promise<WorkspaceCatalog>;
    subscribe(listener: (catalog: WorkspaceCatalog) => void): () => void;
    createWorkspace(name: string, input: WorkspacePayload, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument>;
    saveWorkspace(id: string, input: WorkspacePayload, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument>;
    createTemplate(name: string, input: IndicatorTemplateInput, options?: WorkspaceOperationOptions): Promise<IndicatorTemplateDocument>;
    /** Update reusable study settings without changing the saved template identity. */
    saveTemplate(id: string, input: IndicatorTemplateInput, options?: WorkspaceOperationOptions): Promise<IndicatorTemplateDocument>;
    rename(kind: WorkspaceKind, id: string, name: string, options?: WorkspaceOperationOptions): Promise<void>;
    duplicate(kind: WorkspaceKind, id: string, name: string, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument | IndicatorTemplateDocument>;
    remove(kind: WorkspaceKind, id: string, options?: WorkspaceOperationOptions): Promise<void>;
    openWorkspace(id: string, options?: WorkspaceOpenOptions): Promise<WorkspaceDocument>;
    setAutosave(enabled: boolean, options?: WorkspaceOperationOptions): Promise<void>;
    importDocument(input: unknown, options?: WorkspaceOperationOptions): Promise<WorkspaceDocument | IndicatorTemplateDocument>;
    /** `captureIndicatorTemplate`, as the store member a widget reaches it through. */
    captureIndicatorTemplate(chart: Chart): IndicatorTemplatePayload;
    /** `planIndicatorTemplateState`, as the store member a widget reaches it through. */
    planIndicatorTemplateState(chart: Chart, incoming: IndicatorTemplateInput, mode: IndicatorTemplateMode, options?: IndicatorTemplateApplyOptions): IndicatorTemplatePlan;
    exportDocument(kind: WorkspaceKind, id: string): Promise<string>;
    private _find;
    private _updatedAt;
    private _newId;
    private _insert;
}

/**
 * Revision-checked workspace storage in memory: for tests, previews and hosts
 * without IndexedDB. Nothing outlives the page. `seed` maps namespaces to
 * catalogs. It keeps the IndexedDB adapter's contract: a stale or skipping
 * write is refused, and a corrupt stored catalog is never replaced.
 */
declare function createMemoryWorkspaceStorage(seed?: Readonly<Record<string, unknown>>): WorkspaceStorage;

interface IndexedDbWorkspaceStorage extends WorkspaceStorage {
    /** Release the connection. Existing transactions may finish; new calls reject. */
    close(): Promise<void>;
}
/** A revisioned catalog store: what the workspace, watchlist and drawing-template adapters share. */
interface IndexedDbCatalogStorage<C extends {
    revision: number;
}> {
    read(namespace: string): Promise<unknown | null>;
    write(namespace: string, catalog: C, expectedRevision: number, options?: WorkspaceOperationOptions): Promise<void>;
    close(): Promise<void>;
}
/**
 * Browser persistence with one atomic compare-and-write transaction per catalog.
 * Pass the host's indexedDB explicitly; importing this module needs no browser.
 */
declare function createIndexedDbWorkspaceStorage(factory: IDBFactory, databaseName?: string): IndexedDbWorkspaceStorage;

/** One saved instrument. Both parts are opaque: never case-folded or parsed. */
type WatchlistEntry = InstrumentKey;
interface Watchlist {
    id: string;
    name: string;
    entries: WatchlistEntry[];
    createdAt: number;
    updatedAt: number;
}
interface WatchlistCatalog {
    version: 1;
    revision: number;
    lists: Watchlist[];
    activeListId: string | null;
}
interface WatchlistOperationOptions {
    /** Cancellation is effective until the storage transaction commits. */
    signal?: AbortSignal | undefined;
    /**
     * Refuse the change when the catalog has moved past this revision. Pass the
     * revision a position-based edit (a move) was computed from, so it cannot
     * land on a list another session has reordered.
     */
    expectedRevision?: number;
}
/** The host must atomically reject writes whose expectedRevision is stale. */
interface WatchlistStorage {
    read(namespace: string): Promise<unknown | null>;
    write(namespace: string, catalog: WatchlistCatalog, expectedRevision: number, options?: WatchlistOperationOptions): Promise<void>;
}
interface WatchlistRepositoryOptions {
    now?: () => number;
    id?: () => string;
}
/**
 * What a watchlist panel calls, and nothing more. `WatchlistRepository`
 * implements it, with `duplicateList` besides; a host with its own
 * server-side lists can implement it directly instead.
 */
interface WatchlistStore {
    load(): Promise<WatchlistCatalog>;
    /**
     * Called after each change this store commits, before that change's promise
     * resolves, as `WatchlistRepository` does: the panel computes a following
     * move from the catalog delivered here. Returns the unsubscribe.
     */
    subscribe(listener: (catalog: WatchlistCatalog) => void): () => void;
    createList(name: string, entries?: readonly WatchlistEntry[], options?: WatchlistOperationOptions): Promise<Watchlist>;
    renameList(id: string, name: string, options?: WatchlistOperationOptions): Promise<Watchlist>;
    removeList(id: string, options?: WatchlistOperationOptions): Promise<void>;
    setActiveList(id: string | null, options?: WatchlistOperationOptions): Promise<void>;
    addEntry(id: string, entry: WatchlistEntry, options?: WatchlistOperationOptions & {
        index?: number;
    }): Promise<Watchlist>;
    removeEntry(id: string, entry: WatchlistEntry, options?: WatchlistOperationOptions): Promise<Watchlist>;
    moveEntry(id: string, entry: WatchlistEntry, index: number, options?: WatchlistOperationOptions): Promise<Watchlist>;
}
declare class WatchlistConflictError extends Error {
    constructor();
}
/**
 * The identity of an entry as one string. JSON of the pair rather than a
 * joined "exchange:symbol", because an opaque symbol may itself contain the
 * separator and two different instruments would then share a key.
 */
declare function watchlistKey(entry: InstrumentKey): string;
/** Parse the complete catalog before any mutation; corrupt storage is never replaced. */
declare function parseWatchlistCatalog(input: unknown): WatchlistCatalog;
/**
 * Named symbol lists with serialized, revision-checked writes. Each change is
 * applied to the catalog as stored at that moment, so an edit made in another
 * session survives; only a write racing it is refused, as a conflict. Create a
 * new repository when the account changes.
 */
declare class WatchlistRepository implements WatchlistStore {
    private readonly _catalog;
    private readonly _now;
    private readonly _id;
    constructor(storage: WatchlistStorage, namespace: string, options?: WatchlistRepositoryOptions);
    get namespace(): string;
    load(): Promise<WatchlistCatalog>;
    subscribe(listener: (catalog: WatchlistCatalog) => void): () => void;
    createList(name: string, entries?: readonly WatchlistEntry[], options?: WatchlistOperationOptions): Promise<Watchlist>;
    renameList(id: string, name: string, options?: WatchlistOperationOptions): Promise<Watchlist>;
    duplicateList(id: string, name: string, options?: WatchlistOperationOptions): Promise<Watchlist>;
    removeList(id: string, options?: WatchlistOperationOptions): Promise<void>;
    setActiveList(id: string | null, options?: WatchlistOperationOptions): Promise<void>;
    addEntry(id: string, entry: WatchlistEntry, options?: WatchlistOperationOptions & {
        index?: number;
    }): Promise<Watchlist>;
    removeEntry(id: string, entry: WatchlistEntry, options?: WatchlistOperationOptions): Promise<Watchlist>;
    moveEntry(id: string, entry: WatchlistEntry, index: number, options?: WatchlistOperationOptions): Promise<Watchlist>;
    private _find;
    private _touch;
    private _position;
    private _index;
    private _newId;
}
/**
 * Revision-checked storage in memory: for tests, previews and hosts without
 * IndexedDB. Nothing outlives the page. `seed` maps namespaces to catalogs.
 */
declare function createMemoryWatchlistStorage(seed?: Readonly<Record<string, unknown>>): WatchlistStorage;
type IndexedDbWatchlistStorage = IndexedDbCatalogStorage<WatchlistCatalog> & WatchlistStorage;
/**
 * Browser persistence with one atomic compare-and-write transaction per
 * catalog, the same contract the workspace adapter keeps. Pass the host's
 * indexedDB explicitly; importing this module needs no browser.
 */
declare function createIndexedDbWatchlistStorage(factory: IDBFactory, databaseName?: string): IndexedDbWatchlistStorage;

/** One level of a Fibonacci-family ladder, as `style.levels` holds it. */
interface DrawingTemplateLevel {
    ratio: number;
    color?: string;
    enabled?: boolean;
    label?: string;
}
/** A settings value: what a schema field of any kind stores. */
type DrawingTemplateValue = string | number | boolean | DrawingTemplateLevel[];
/** Settings by schema path, as `readDrawingSettings` returns them. */
type DrawingTemplateValues = Record<string, DrawingTemplateValue>;
/** A saved look for one tool. */
interface DrawingTemplate {
    id: string;
    name: string;
    /** The drawing tool id the template was saved from, and the only tool it is offered for. */
    tool: string;
    values: DrawingTemplateValues;
    createdAt: number;
    updatedAt: number;
}
/** The look a tool's new drawings start with. At most one per tool. */
interface DrawingToolDefault {
    tool: string;
    values: DrawingTemplateValues;
    updatedAt: number;
}
interface DrawingTemplateCatalog {
    version: 1;
    revision: number;
    templates: DrawingTemplate[];
    defaults: DrawingToolDefault[];
}
interface DrawingTemplateOperationOptions {
    /** Cancellation is effective until the storage transaction commits. */
    signal?: AbortSignal | undefined;
    /** Refuse the change when the catalog has moved past this revision. */
    expectedRevision?: number;
}
/** The host must atomically reject writes whose expectedRevision is stale. */
interface DrawingTemplateStorage {
    read(namespace: string): Promise<unknown | null>;
    write(namespace: string, catalog: DrawingTemplateCatalog, expectedRevision: number, options?: DrawingTemplateOperationOptions): Promise<void>;
}
interface DrawingTemplateRepositoryOptions {
    now?: () => number;
    id?: () => string;
}
/**
 * What the widget calls, and nothing more. `DrawingTemplateRepository`
 * implements it; a host that keeps templates on its own server can implement
 * it directly instead.
 */
interface DrawingTemplateStore {
    load(): Promise<DrawingTemplateCatalog>;
    /** Called after each change this store commits, before that change's promise resolves. Returns the unsubscribe. */
    subscribe(listener: (catalog: DrawingTemplateCatalog) => void): () => void;
    /**
     * Save `values` under `name` for `tool`. A template of the same tool whose
     * name matches (ignoring case and outer spaces) takes the new values and
     * keeps its identity, so saving twice under one name never makes two.
     */
    saveTemplate(name: string, tool: string, values: DrawingTemplateValues, options?: DrawingTemplateOperationOptions): Promise<DrawingTemplate>;
    renameTemplate(id: string, name: string, options?: DrawingTemplateOperationOptions): Promise<DrawingTemplate>;
    removeTemplate(id: string, options?: DrawingTemplateOperationOptions): Promise<void>;
    /** The look new drawings of `tool` start with; null forgets it, and the tool's own look returns. */
    setDefault(tool: string, values: DrawingTemplateValues | null, options?: DrawingTemplateOperationOptions): Promise<void>;
}
declare class DrawingTemplateConflictError extends Error {
    constructor();
}
/** Validate a template's values: bounded, schema-shaped paths, no drawing content. */
declare function parseDrawingTemplateValues(input: unknown): DrawingTemplateValues;
/** Parse the complete catalog before any mutation; corrupt storage is never replaced. */
declare function parseDrawingTemplateCatalog(input: unknown): DrawingTemplateCatalog;
/**
 * Drawing templates and tool defaults with serialized, revision-checked
 * writes. Each change is applied to the catalog as stored at that moment, so
 * a template saved in another session survives; only a write racing it is
 * refused, as a conflict. Create a new repository when the account changes.
 */
declare class DrawingTemplateRepository implements DrawingTemplateStore {
    private readonly _catalog;
    private readonly _now;
    private readonly _id;
    constructor(storage: DrawingTemplateStorage, namespace: string, options?: DrawingTemplateRepositoryOptions);
    get namespace(): string;
    load(): Promise<DrawingTemplateCatalog>;
    subscribe(listener: (catalog: DrawingTemplateCatalog) => void): () => void;
    saveTemplate(name: string, tool: string, values: DrawingTemplateValues, options?: DrawingTemplateOperationOptions): Promise<DrawingTemplate>;
    renameTemplate(id: string, name: string, options?: DrawingTemplateOperationOptions): Promise<DrawingTemplate>;
    removeTemplate(id: string, options?: DrawingTemplateOperationOptions): Promise<void>;
    setDefault(tool: string, values: DrawingTemplateValues | null, options?: DrawingTemplateOperationOptions): Promise<void>;
    private _find;
    private _later;
    private _newId;
}
/**
 * Revision-checked storage in memory: for tests, previews and hosts without
 * IndexedDB. Nothing outlives the page. `seed` maps namespaces to catalogs.
 */
declare function createMemoryDrawingTemplateStorage(seed?: Readonly<Record<string, unknown>>): DrawingTemplateStorage;
type IndexedDbDrawingTemplateStorage = IndexedDbCatalogStorage<DrawingTemplateCatalog> & DrawingTemplateStorage;
/**
 * Browser persistence with one atomic compare-and-write transaction per
 * catalog, the contract the workspace and watchlist adapters keep. Pass the
 * host's indexedDB explicitly; importing this module needs no browser.
 */
declare function createIndexedDbDrawingTemplateStorage(factory: IDBFactory, databaseName?: string): IndexedDbDrawingTemplateStorage;

export { type DrawingTemplate, type DrawingTemplateCatalog, DrawingTemplateConflictError, type DrawingTemplateLevel, type DrawingTemplateOperationOptions, DrawingTemplateRepository, type DrawingTemplateRepositoryOptions, type DrawingTemplateStorage, type DrawingTemplateStore, type DrawingTemplateValue, type DrawingTemplateValues, type DrawingToolDefault, type IndexedDbCatalogStorage, type IndexedDbDrawingTemplateStorage, type IndexedDbWatchlistStorage, type IndexedDbWorkspaceStorage, type IndicatorTemplateApplyOptions, type IndicatorTemplateDocument, type IndicatorTemplateInput, type IndicatorTemplateLayout, type IndicatorTemplateMode, type IndicatorTemplatePayload, type IndicatorTemplatePlan, type IndicatorTemplatePlotBinding, WORKSPACE_VERSION, type Watchlist, type WatchlistCatalog, WatchlistConflictError, type WatchlistEntry, type WatchlistOperationOptions, WatchlistRepository, type WatchlistRepositoryOptions, type WatchlistStorage, type WatchlistStore, type WorkspaceCatalog, type WorkspaceChartState, type WorkspaceComparison, WorkspaceConflictError, type WorkspaceDocument, WorkspaceDocumentError, type WorkspaceKind, type WorkspaceLinkChannels, type WorkspaceLinkGroup, type WorkspaceOpenOptions, type WorkspaceOperationOptions, type WorkspacePane, type WorkspacePayload, WorkspaceRepository, type WorkspaceRepositoryOptions, type WorkspaceSettings, type WorkspaceSlot, type WorkspaceStorage, type WorkspaceStore, type WorkspaceSync, captureIndicatorTemplate, createIndexedDbDrawingTemplateStorage, createIndexedDbWatchlistStorage, createIndexedDbWorkspaceStorage, createMemoryDrawingTemplateStorage, createMemoryWatchlistStorage, createMemoryWorkspaceStorage, migrateWidgetWorkspace, parseDrawingTemplateCatalog, parseDrawingTemplateValues, parseIndicatorStates, parseIndicatorTemplate, parseIndicatorTemplatePayload, parseWatchlistCatalog, parseWorkspaceCatalog, parseWorkspaceDocument, parseWorkspacePayload, planIndicatorTemplate, planIndicatorTemplateState, watchlistKey };
