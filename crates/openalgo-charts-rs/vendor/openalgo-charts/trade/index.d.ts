import { IPrimitive, PrimitiveHost, ZOrder, PrimitiveRenderContext, PrimitiveHit, TradingCapabilityResult, TickSchedule, TradingCapabilitySource, Instrument } from 'openalgo-charts';
export { TradingCapabilities, TradingCapabilityError, TradingCapabilityRequest, TradingCapabilityResult, TradingCapabilitySource, TradingOperation, assertTradingCapability, checkTradingCapability } from 'openalgo-charts';

/**
 * Internal time is always **UTC seconds** (integer). Feed adapters convert
 * broker formats (IST strings, epoch ms) to this at the edge; see ARCHITECTURE.md §4.0.
 */
type UTCSeconds = number;

interface DepthLevel {
    price: number;
    qty: number;
    orders?: number;
}
/** Variable-depth book; `bids`/`asks` length = whatever the broker streams (5..200). */
interface MarketDepth {
    /** Exchange event timestamp in UTC seconds, when supplied by the feed. */
    timeSec?: UTCSeconds;
    bids: DepthLevel[];
    asks: DepthLevel[];
    ltp: number;
    ltq?: number;
}
/** The side of an order, one declaration for the base and the trade tier. */
type OrderSide = 'BUY' | 'SELL';
/** How an order is priced, one declaration for the base and the trade tier. */
type OrderType = 'MARKET' | 'LIMIT' | 'SL' | 'SL-M';

/**
 * Trade-layer data model (ARCHITECTURE.md §9). Broker-agnostic shapes a broker
 * adapter produces; the chart depends only on these, not on OpenAlgo's REST.
 */

/** Lifecycle states (§9.5): the trade controller reconciles them from book snapshots, and the order engine moves them as it writes. */
type OrderStatus = 'pending' | 'working' | 'partial' | 'filled' | 'cancelled' | 'rejected';
type OrderRole = 'entry' | 'sl' | 'tp';
interface Order {
    id: string;
    symbol: string;
    side: OrderSide;
    type: OrderType;
    qty: number;
    filledQty: number;
    price: number;
    triggerPrice?: number | undefined;
    status: OrderStatus;
    /** Links SL/TP child orders to their position/entry. */
    parentId?: string;
    role?: OrderRole | undefined;
}
interface Position {
    symbol: string;
    /** Net signed quantity: positive = long, negative = short, 0 = flat. */
    netQty: number;
    avgPrice: number;
    /** The account holding it, when the provider says. `AccountManager.positions()` drops a row naming another account. */
    accountId?: string;
}
/** A working order is one still live in the book (not terminal). */
declare function isWorking(o: Order): boolean;

/**
 * P&L and risk math (ARCHITECTURE.md §9.1). Pure functions: the hot path on
 * every LTP tick, and the part most worth unit-testing.
 */

/** Unrealized P&L for a position at the given last price. */
declare function unrealizedPnl(position: Position, ltp: number): number;
/** Unrealized P&L as a percentage of the entry notional. */
declare function unrealizedPnlPercent(position: Position, ltp: number): number;
/** Breakeven price (entry ± per-unit charges; charges default 0). */
declare function breakeven(position: Position, chargesPerUnit?: number): number;
/** Risk:reward for a bracket relative to entry. Returns null if risk is zero. */
declare function riskReward(entry: number, stop: number, target: number): number | null;
/** True if a stop/target is correctly placed for the side (long: SL<entry<TP). */
declare function bracketValid(side: 'BUY' | 'SELL', entry: number, stop: number, target: number): boolean;

/**
 * Working-order line (ARCHITECTURE.md §9.1). A horizontal dashed line at the
 * order price, colored by side, with a broker-style segmented pill group on the
 * line ([SIDE][qty][TYPE price ±LTP-distance][close]) and a compact price tag on
 * the axis. Pending (un-acked) orders draw dimmed; partial fills show progress
 * (3/10). Hover thickens the line (hit-test `order:<id>`, ns-resize cursor for
 * drag-to-modify; the close segment hit-tests as `order:<id>::close`).
 */

declare class WorkingOrderLine implements IPrimitive {
    private _order;
    private _ltp;
    private _host;
    private _group;
    private readonly _extent;
    /** Pre-drag price, self-captured on the first dragged frame (drag ghost). */
    private _ghost;
    constructor(order: Order, options?: {
        extentFromRight?: number;
    });
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    get order(): Order;
    update(order: Order): void;
    setLtp(ltp: number): void;
    autoscaleInfo(): {
        min: number;
        max: number;
    };
    private _price;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
}

/**
 * Position marker (ARCHITECTURE.md §9.1). A line at the average entry price
 * with a broker-style segmented pill group ([LONG|SHORT][qty][P&L (pct)][close]),
 * a compact avg-price tag on the axis, and a shaded entry-to-LTP band colored by
 * P&L sign. Updates cheaply on every LTP tick. The close segment hit-tests as
 * `position:<symbol>::close` (wire it to your square-off flow).
 */

declare class PositionMarker implements IPrimitive {
    private _position;
    private _ltp;
    private _host;
    private _group;
    private readonly _extent;
    constructor(position: Position, options?: {
        extentFromRight?: number;
    });
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    get position(): Position;
    update(position: Position): void;
    setLtp(ltp: number): void;
    autoscaleInfo(): {
        min: number;
        max: number;
    };
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
}

/**
 * Bracket group (ARCHITECTURE.md §9.1). SL + Target lines tied to a position,
 * with theme-derived shaded risk/reward zones, rounded "SL/TP @ price" chips at
 * the left edge, and an R:R chip near entry: the core advanced-trade-management
 * visualisation. The SL/TP lines thicken on hover and are draggable (OCO modify).
 */

interface BracketState {
    symbol: string;
    side: OrderSide;
    entry: number;
    stop: number;
    target: number;
}
declare class BracketGroup implements IPrimitive {
    private _state;
    private _host;
    constructor(state: BracketState);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    get state(): BracketState;
    update(state: BracketState): void;
    autoscaleInfo(): {
        min: number;
        max: number;
    };
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
}

/**
 * Trade controller (ARCHITECTURE.md §9.3), read-only by design: the order
 * engine owns the write path. The single
 * source of truth: it reconciles order/position book snapshots into on-chart
 * primitives (add/update/remove) and pushes LTP into them for live P&L. On a
 * reconnect, a fresh snapshot is diffed against current primitives, so vanished
 * orders are removed (STALE handling) with no special code path.
 */

/** Where the controller attaches/detaches its primitives (the chart implements this). */
interface TradeHost {
    addPrimitive(p: IPrimitive): void;
    removePrimitive(p: IPrimitive): void;
}
declare class TradeController {
    private readonly _host;
    private readonly _orderLines;
    private readonly _markers;
    private readonly _brackets;
    private readonly _ltp;
    constructor(host: TradeHost);
    /** Reconcile a full book snapshot. Idempotent: safe to call on every update or reconnect. */
    reconcile(orders: readonly Order[], positions: readonly Position[]): void;
    private _reconcileOrderLines;
    private _reconcilePositions;
    /**
     * Decide which brackets to draw, without touching the host. `covered` names the
     * exact orders those brackets account for; every other working order still
     * needs a line of its own.
     */
    private _planBrackets;
    private _applyBrackets;
    /** Push a last price; updates the live P&L / distance on bound primitives. */
    onLtp(symbol: string, ltp: number): void;
    /** Test/introspection helpers. */
    orderLineCount(): number;
    positionCount(): number;
    bracketCount(): number;
}

/**
 * Order state machine (ARCHITECTURE.md §9.5). Explicit client-side states with
 * a guarded transition table: no optimistic guesswork. Pure and fully testable.
 *
 *   pending_place -ack-> working -fill-> filled
 *                 -reject-> rejected
 *   working/partial -reject-> rejected     (the broker refusing one it accepted)
 *   working/partial -submitModify-> modify_pending -ack-> working / -reject-> working
 *   working/partial -submitCancel-> cancel_pending -cancelled-> cancelled / -reject-> working
 *   any non-terminal -reconnectAbsent-> stale
 */
type ClientOrderState = 'pending_place' | 'working' | 'partial' | 'filled' | 'modify_pending' | 'cancel_pending' | 'rejected' | 'cancelled' | 'stale';
type OrderEvent = 'ack' | 'partialFill' | 'fill' | 'reject' | 'submitModify' | 'submitCancel' | 'cancelled' | 'reconnectAbsent';
declare function isTerminal(state: ClientOrderState): boolean;
/** Whether `event` is allowed from `state`. */
declare function canTransition(state: ClientOrderState, event: OrderEvent): boolean;
/** Apply `event`; returns the next state, or the same state if the event is invalid. */
declare function transition(state: ClientOrderState, event: OrderEvent): ClientOrderState;

/**
 * Declarations for the trading operations that came after place, modify and
 * cancel: account reads, order preview, durations, leverage, and the native
 * position commands.
 *
 * The rule here is the opposite of the original capability flags. Those treat
 * an omission as "the legacy route still works", because a provider written
 * before they existed must keep placing orders. Nothing written before this
 * file can carry an account, a duration or a close, so an omission here means
 * unsupported. A host that assumed otherwise would drop the field on the wire
 * and send the order somewhere the user did not choose, or dress an opposite
 * order up as a native close.
 */

/** Time in force for a new order. `GTD` needs an expiry. */
type OrderDuration = 'DAY' | 'IOC' | 'FOK' | 'GTC' | 'GTD';
/** Every duration the request schema understands, in display order. */
declare const ORDER_DURATIONS: readonly OrderDuration[];
type TradingFeature = 'accounts' | 'executions' | 'orderHistory' | 'preview' | 'duration' | 'leverage' | 'close' | 'partialClose' | 'reverse' | 'brackets';
/** What a provider declares. Omitted, false and `'unknown'` all refuse. */
interface TradingFeatures {
    /** Account list, snapshots, positions and selection. */
    readonly accounts?: boolean | 'unknown';
    readonly executions?: boolean | 'unknown';
    readonly orderHistory?: boolean | 'unknown';
    readonly preview?: boolean | 'unknown';
    /** Durations accepted on new orders. Omitted: no explicit duration may be sent. */
    readonly durations?: readonly OrderDuration[];
    readonly leverage?: boolean | 'unknown';
    readonly close?: boolean | 'unknown';
    readonly partialClose?: boolean | 'unknown';
    readonly reverse?: boolean | 'unknown';
    /** Brackets whose stop and target legs the provider itself links. */
    readonly brackets?: boolean | 'unknown';
}
interface TradingFeatureRequest {
    readonly feature: TradingFeature;
    readonly symbol?: string | undefined;
    readonly exchange?: string | undefined;
    readonly account?: string | undefined;
    readonly mode?: 'live' | 'analyzer' | undefined;
    readonly type?: OrderType | undefined;
    readonly duration?: OrderDuration | undefined;
}
/** A configured provider returning undefined, throwing or answering late declares nothing. */
type TradingFeatureSource = TradingFeatures | ((request: Readonly<TradingFeatureRequest>) => TradingFeatures | undefined);
/** The words a refusal uses for a feature, shared so every surface says the same thing. */
declare function tradingFeatureLabel(feature: TradingFeature): string;
/** Shared by host controls, the order engine, the account manager and the simulated broker. */
declare function checkTradingFeature(source: TradingFeatureSource | undefined, request: TradingFeatureRequest): TradingCapabilityResult;

/**
 * Read-only account state: which accounts a provider offers, the selected
 * one's balances and margin, its positions, executions and order history.
 *
 * Nothing in this file writes. Selecting an account changes what the host
 * reads and which account a later order names; it never sends, re-routes or
 * cancels an order. Three hazards shape the rest:
 *
 * - **A late answer for the old account.** Switching from A to B while A's
 *   snapshot is in flight must not paint A's equity under B's name. Every
 *   selection bumps a generation, aborts the old request and unsubscribes the
 *   old stream, and every answer is checked against the generation, the
 *   account id and the reading's own time before it is shown.
 * - **The other ledger.** An analyzer (sandbox) view must never show a live
 *   balance, or the reverse. Accounts of the other mode are not listed, and a
 *   snapshot that names the other mode is refused, not displayed.
 * - **A figure nobody could read.** A non-finite balance is reported as an
 *   unreadable snapshot. Zero would be a confident, wrong number.
 */

interface TradingAccount {
    readonly id: string;
    readonly name?: string;
    /** The ledger the account belongs to. */
    readonly mode: TradeMode;
    readonly currency?: string;
}
/** One reading of an account. Money fields are in `currency`; absent means the provider did not say. */
interface AccountSnapshot {
    readonly accountId: string;
    readonly mode: TradeMode;
    /** When the provider took the reading, UTC seconds (fractions allowed). Orders readings. */
    readonly asOf: number;
    readonly currency?: string;
    readonly balance?: number;
    readonly equity?: number;
    readonly marginUsed?: number;
    readonly marginAvailable?: number;
    readonly unrealizedPnl?: number;
    readonly realizedPnl?: number;
    readonly leverage?: number;
}
interface Execution {
    readonly id: string;
    readonly accountId: string;
    readonly orderId: string;
    readonly symbol: string;
    readonly exchange?: string;
    readonly side: OrderSide;
    readonly qty: number;
    readonly price: number;
    /** UTC seconds, the unit bar times use, so a fill can be placed on its bar. */
    readonly time: number;
}
interface OrderHistoryEntry {
    readonly accountId: string;
    /** The order as the provider last reported it. */
    readonly order: Order;
    /** Last update, UTC seconds. */
    readonly time: number;
    /** The client token the provider echoes, which is how an ambiguous write is found again. */
    readonly clientToken?: string;
    readonly duration?: OrderDuration;
    /** UTC seconds. */
    readonly expiresAt?: number;
    /** Set when the order is a provider-native command rather than a plain order. */
    readonly command?: 'close' | 'reverse' | 'bracket';
}
interface AccountHistoryQuery {
    readonly accountId: string;
    readonly symbol?: string;
    /** Inclusive bounds, UTC seconds. */
    readonly from?: number;
    readonly to?: number;
    readonly limit?: number;
}
/**
 * The read half of a broker. Every member is optional because providers vary;
 * `features` says which are real, and a method without a declaration is unused.
 */
interface AccountFeed {
    readonly features?: TradingFeatureSource | undefined;
    listAccounts?(signal: AbortSignal): Promise<readonly TradingAccount[]>;
    getAccountSnapshot?(accountId: string, signal: AbortSignal): Promise<AccountSnapshot>;
    /** Push readings as they change. `onError` reports a dead stream; the returned function stops it. */
    subscribeAccount?(accountId: string, onSnapshot: (snapshot: AccountSnapshot) => void, onError?: (error: unknown) => void): () => void;
    getAccountPositions?(accountId: string, signal: AbortSignal): Promise<readonly Position[]>;
    getExecutions?(query: AccountHistoryQuery, signal: AbortSignal): Promise<readonly Execution[]>;
    getOrderHistory?(query: AccountHistoryQuery, signal: AbortSignal): Promise<readonly OrderHistoryEntry[]>;
}
/**
 * `unsupported`: the provider or host declares no account data (see `reason`).
 * `idle`: supported, nothing loaded yet. `stale`: the last figures are kept but
 * may be out of date. `error`: nothing trustworthy to show.
 */
type AccountStatus = 'unsupported' | 'idle' | 'loading' | 'ready' | 'stale' | 'error';
interface AccountState {
    readonly status: AccountStatus;
    readonly reason?: string;
    readonly mode: TradeMode;
    readonly accounts: readonly TradingAccount[];
    readonly selectedId: string | null;
    readonly snapshot: AccountSnapshot | null;
    /** Bumped by every selection, refresh and disconnect, so a reader can tell epochs apart. */
    readonly generation: number;
}
type AccountSelectResult = {
    ok: true;
    snapshot: AccountSnapshot;
} | {
    ok: false;
    reason: string;
    cancelled?: boolean;
};
type AccountReadResult<T> = {
    ok: true;
    accountId: string;
    rows: readonly T[];
    dropped: number;
} | {
    ok: false;
    reason: string;
    cancelled?: boolean;
    unsupported?: boolean;
};
/** What a host control needs to show and switch accounts. `AccountManager` implements it. */
interface AccountStateSource {
    getState(): AccountState;
    subscribe(listener: (state: AccountState) => void): () => void;
    select(accountId: string): Promise<AccountSelectResult>;
}
interface AccountManagerOptions {
    feed: AccountFeed;
    /** The ledger this manager shows. Accounts of the other mode are never listed. Default `live`. */
    mode?: TradeMode;
    /** Host restrictions combined with the feed's declaration; either can refuse. */
    features?: TradingFeatureSource;
    /** Selected after the first listing when the provider offers it. Default: the first account. */
    initialAccount?: string;
}
declare class AccountManager implements AccountStateSource {
    private readonly _feed;
    private readonly _mode;
    private readonly _hostFeatures?;
    private readonly _initial?;
    private readonly _listeners;
    /** Aborted together whenever the generation moves on. */
    private _aborts;
    private _unsubscribe;
    private _state;
    private _destroyed;
    constructor(options: AccountManagerOptions);
    get mode(): TradeMode;
    getState(): AccountState;
    selectedAccount(): string | null;
    subscribe(listener: (state: AccountState) => void): () => void;
    /** Null when the feature is usable, otherwise why not. */
    private _support;
    private _set;
    /** Move to a new epoch: abort everything in flight and drop the stream. */
    private _advance;
    private _controller;
    /** List accounts, keep or choose a selection, and load its snapshot. */
    refresh(): Promise<AccountSelectResult>;
    /** Select an account and load its snapshot. A later selection cancels this one. */
    select(accountId: string): Promise<AccountSelectResult>;
    private _load;
    /** Validate a reading for `accountId`; a string explains the refusal. */
    private _accept;
    private _stream;
    /**
     * The provider or the host lost the connection. The last figures stay on
     * screen marked stale; anything still in flight is abandoned, because it was
     * asked for over the connection that just failed.
     */
    disconnected(reason?: string): void;
    /** Re-list and reload after a disconnect. The same as `refresh`, named for the call site. */
    reconnect(): Promise<AccountSelectResult>;
    /**
     * A position row carries its account only when the provider stamps one. A
     * row naming another account is dropped and counted; one naming none is the
     * selected account's, because that is the account the provider was asked for.
     */
    positions(): Promise<AccountReadResult<Position>>;
    executions(query?: Omit<AccountHistoryQuery, 'accountId'>): Promise<AccountReadResult<Execution>>;
    orderHistory(query?: Omit<AccountHistoryQuery, 'accountId'>): Promise<AccountReadResult<OrderHistoryEntry>>;
    /**
     * One read for the selected account. Rows for any other account are dropped
     * and counted, not merged: a provider that answers with a neighbour's fills
     * is wrong, and showing them would be worse than showing fewer.
     */
    private _read;
    /** Abort everything, drop the stream and the listeners. Later calls refuse. */
    destroy(): void;
}

/**
 * Pre-trade validation (ARCHITECTURE.md §9.5). Pure guards run client-side so a
 * bad order is blocked and explained rather than round-tripping to a broker
 * rejection. Tick-size rounding, price-band, quantity and freeze-quantity checks.
 *
 * ADVISORY ONLY. The broker RMS is authoritative for every rule here. A check
 * that a user can bypass by editing the call in devtools is a UX affordance, not
 * a risk control: it exists to explain a mistake before the round trip, never to
 * be the thing that stops a bad order. Anything that must actually hold has to
 * live in the feed (see the OpenAlgo compatibility note: the production terminal
 * calls `trade.place` directly and never constructs `OrderEngine`).
 */

interface PriceBand {
    lower: number;
    upper: number;
}
/**
 * Machine-readable rejection code. Three of these (`QTY_STEP`,
 * `QTY_FREEZE_LIMIT`, `PRICE_OUT_OF_BAND`) are spelled exactly as the planned v2
 * broker contract's `RejectCode` members, so a mapping to it would be
 * identity; `QTY_INVALID` and `PRICE_INVALID` are local, and would fold into
 * its `CONTRACT_VIOLATION`. Every member is emitted by a branch below: a code
 * nothing can produce would be a label with nothing behind it.
 */
type ValidationCode = 'QTY_INVALID' | 'QTY_STEP' | 'QTY_FREEZE_LIMIT' | 'PRICE_INVALID' | 'PRICE_OUT_OF_BAND';
interface ValidationResult {
    ok: boolean;
    reason?: string;
    /** Machine-readable form of `reason`; present on every rejection. */
    code?: ValidationCode;
    /** Price after tick-size snapping (when applicable). */
    price?: number;
}
interface OrderConstraints {
    tickSize: number;
    /**
     * Price-dependent ticks. When set it decides every snap and `tickSize` is
     * only read by older code, so give it the schedule's `minMove`. Absent or
     * null, the constant `tickSize` snaps exactly as it always has.
     */
    tickSchedule?: TickSchedule | null;
    priceBand?: PriceBand;
    /** Max quantity per single order (exchange freeze limit). */
    freezeQty?: number;
    /**
     * Contract lot size. When set, quantity must be a whole multiple of it: 100 on
     * a 75-lot future is rejected here rather than by the exchange.
     */
    lotSize?: number;
    /**
     * Set true only for a venue that genuinely trades fractions, such as crypto.
     * Absent means whole quantities, which is every Indian equity and F&O contract
     * and the reason fractional quantities are rejected by default: a fractional
     * quantity reaching those brokers is a defect, not a preference. A `lotSize`
     * grid still applies when this is true.
     */
    allowFractionalQty?: boolean;
}
/** True if `price` lies within the inclusive band. */
declare function withinPriceBand(price: number, band: PriceBand): boolean;
/**
 * Validate a quantity on its own. Split out from `validateOrder` because a
 * market order has no price, and the freeze and lot limits must still apply to
 * it: routing quantity checks through the price path left them unreachable for
 * exactly the orders that are hardest to take back.
 */
declare function validateQuantity(qty: number, c: OrderConstraints): ValidationResult;
/**
 * Validate a price on its own, returning the tick-snapped value. Snapping is
 * unchanged; the only addition is rejecting a non-finite price, which otherwise
 * snaps to NaN and then passes an absent band as if it were a real level.
 */
declare function validatePrice(price: number, c: OrderConstraints): ValidationResult;
/**
 * Validate a price + qty against constraints. Snaps price to the tick size and
 * returns the snapped value; rejects out-of-band prices and bad quantities.
 *
 * `price` may be omitted for an order that has none (market), in which case only
 * the quantity is checked and no `price` comes back. Quantity is checked first,
 * so a request wrong in both ways reports the quantity, as it always has.
 */
declare function validateOrder(price: number | undefined, qty: number, c: OrderConstraints): ValidationResult;

/**
 * The order engine's public contract (order-engine.ts): the requests a host
 * sends, the receipts and results it gets back, the feed the engine drives,
 * the intents an order moves through, the engine's options, and the two
 * checks that tell what kind of failure a write met. Its own module so the
 * engine stays under the line limit; order-engine.ts re-exports all of it.
 */

interface PlaceRequest {
    symbol: string;
    exchange?: string | undefined;
    side: OrderSide;
    type: OrderType;
    qty: number;
    price?: number | undefined;
    triggerPrice?: number | undefined;
    /** Product: CNC (delivery), NRML (F&O carry), MIS (intraday). Required by OpenAlgo. */
    product?: 'CNC' | 'NRML' | 'MIS' | undefined;
    /** Idempotency token; a retry with the same token is never double-sent. */
    clientToken?: string | undefined;
    /** The account the order is for. Needs the `accounts` feature; never dropped on the way to the wire. */
    account?: string;
    /** Time in force. Omitted leaves the provider's own default. Needs the provider to list it. */
    duration?: OrderDuration;
    /** When a `GTD` order lapses, UTC seconds. Only with `GTD`. */
    expiresAt?: number;
    /** Margin multiplier to request. Needs the `leverage` feature. */
    leverage?: number;
}
/** What a provider says an order would cost before it is placed. Absent fields were not reported. */
interface OrderPreview {
    readonly accountId?: string;
    readonly estimatedPrice?: number;
    readonly estimatedValue?: number;
    readonly marginRequired?: number;
    readonly marginAvailableAfter?: number;
    readonly fees?: number;
    readonly currency?: string;
    readonly warnings?: readonly string[];
    /** Set when the provider would refuse the order, and why. */
    readonly rejectReason?: string;
    /** UTC seconds. */
    readonly asOf?: number;
}
type PreviewResult = {
    ok: true;
    preview: OrderPreview;
    request: PlaceRequest;
} | {
    ok: false;
    reason: string;
    unsupported?: boolean;
    stale?: boolean;
};
/** Identifies the position a command acts on. */
interface PositionCommandRequest {
    symbol: string;
    exchange?: string;
    product?: 'CNC' | 'NRML' | 'MIS';
    account?: string;
    /** Idempotency token for this command alone. */
    clientToken?: string;
}
interface ClosePositionRequest extends PositionCommandRequest {
    /** How much to close. Omitted closes the whole position; a quantity is a partial close. */
    qty?: number;
}
type ReversePositionRequest = PositionCommandRequest;
/** An entry whose stop and target legs the provider places and links itself. */
interface BracketOrderRequest extends PlaceRequest {
    /** Trigger of the protective stop leg. */
    stopLoss: number;
    /** Limit price of the target leg. */
    takeProfit: number;
}
/** The broker's handle on a position command. `commandId` is what its order stream reports on. */
interface CommandReceipt {
    commandId: string;
    orderIds?: readonly string[];
}
interface BracketReceipt {
    orderId: string;
    stopLossId?: string;
    takeProfitId?: string;
}
type TradingCommandKind = 'close' | 'reverse' | 'bracket';
/** What a command confirmation is asked to approve. The request carries the account it will use. */
type TradingCommand = {
    kind: 'close';
    request: Readonly<ClosePositionRequest>;
} | {
    kind: 'reverse';
    request: Readonly<ReversePositionRequest>;
} | {
    kind: 'bracket';
    request: Readonly<BracketOrderRequest>;
};
type OrderKind = 'order' | TradingCommandKind | 'bracket-stop' | 'bracket-target';
/**
 * One authoritative order row from the broker, with the client token it echoes
 * when it has one. An `Order` row spreads into it as it is.
 */
interface BrokerOrderUpdate {
    id: string;
    clientToken?: string;
    status: OrderStatus;
    /** For a leg the provider placed and linked itself: the broker id of its entry. */
    parentId?: string;
    /** For such a leg: `sl` for the stop, `tp` for the target. With `parentId`, it finds a leg that echoes no token. */
    role?: OrderRole;
}
/** Fields a modify may change. Whole-order feeds fill the rest from their cache. */
interface ModifyPatch {
    price?: number;
    triggerPrice?: number;
    qty?: number;
}
/**
 * The minimal broker write interface the `OrderEngine` drives: place / modify /
 * cancel. `OpenAlgoTradeFeed` implements this. Distinct from the base-tier
 * `TradeFeed` (a higher-level place + subscribe shape in `openalgo-charts`):
 * implement `OrderFeed` for the engine's write path.
 */
interface OrderFeed {
    /** Optional support declaration; a configured provider can report unavailable metadata. */
    readonly capabilities?: TradingCapabilitySource | undefined;
    /** Declares preview, durations, leverage, accounts and the position commands. Omitted declares none. */
    readonly features?: TradingFeatureSource | undefined;
    place(req: PlaceRequest & {
        mode: TradeMode;
    }): Promise<{
        orderId: string;
    }>;
    modify(orderId: string, patch: ModifyPatch): Promise<void>;
    cancel(orderId: string): Promise<void>;
    /** Read-only: what the order would cost. Must not place anything. */
    previewOrder?(req: PlaceRequest & {
        mode: TradeMode;
    }): Promise<OrderPreview>;
    closePosition?(req: ClosePositionRequest & {
        mode: TradeMode;
    }): Promise<CommandReceipt>;
    reversePosition?(req: ReversePositionRequest & {
        mode: TradeMode;
    }): Promise<CommandReceipt>;
    /**
     * `legClientTokens` are the client tokens the engine gives the stop and
     * target legs. A provider that echoes them on the legs' rows lets a bracket
     * whose answer was lost be reconciled leg by leg.
     */
    placeBracket?(req: BracketOrderRequest & {
        mode: TradeMode;
        legClientTokens: {
            stopLoss: string;
            takeProfit: string;
        };
    }): Promise<BracketReceipt>;
}
type TradeMode = 'live' | 'analyzer';
type GateFn = (req: PlaceRequest) => boolean | Promise<boolean>;
/**
 * Client-owned lifecycle of an intent, kept apart from the broker's own view.
 *
 *   BLOCKED            never sent: validation failed, the gate declined, or the
 *                      feed proved the request never left
 *   SUBMITTING         in flight
 *   SUBMITTED          transport returned success. NOT an order yet.
 *   AMBIGUOUS          transport failed, or a fresh book does not mention it.
 *                      May or may not be live at the exchange. Absorbing until
 *                      the broker speaks.
 *   ACKNOWLEDGED       the broker has accounted for it
 *   MODIFY_SUBMITTING  a modify is in flight
 *   CANCEL_SUBMITTING  a cancel is in flight
 *   RECONCILING        our picture may be behind; a snapshot is being fetched
 *   SETTLED            the broker reported a final state
 */
type IntentState = 'BLOCKED' | 'SUBMITTING' | 'SUBMITTED' | 'AMBIGUOUS' | 'ACKNOWLEDGED' | 'MODIFY_SUBMITTING' | 'CANCEL_SUBMITTING' | 'RECONCILING' | 'SETTLED';
/**
 * What a feed throws when the request PROVABLY never left: it failed before the
 * socket was written (bad arguments, no context cached, offline, DNS). Only this
 * marker releases an idempotency token, because only this proves there is
 * nothing live to double up on. See the catch in `placeOrder`.
 */
interface PreflightFailure {
    readonly preflight: true;
}
/** True when a thrown value declares itself a pre-flight failure. */
declare function isPreflightFailure(err: unknown): boolean;
/**
 * What a feed throws when the broker ANSWERED and refused: the request arrived
 * and was turned down, so nothing is live. That is an authoritative outcome,
 * unlike a transport failure, and it settles the intent as rejected. The token
 * stays claimed because the request did leave; a retry is a new decision.
 */
interface BrokerRejection {
    readonly rejected: true;
}
/** True when a thrown value is the broker's explicit refusal rather than a lost answer. */
declare function isBrokerRejection(err: unknown): boolean;
/** Extra fields for the one-click market order, which a chart button cannot otherwise set. */
interface MarketOrderOptions {
    exchange?: string;
    product?: 'CNC' | 'NRML' | 'MIS';
    /** Idempotency token, so a double-clicked button places one order, not two. */
    clientToken?: string;
}
interface ModifyOptions {
    /**
     * Explicit stop trigger. Omitted, the trigger follows the order type: SL-M
     * treats the dragged level as the trigger, SL carries its trigger at the
     * offset the order was placed with, and the rest have none.
     */
    triggerPrice?: number;
}
interface OrderEngineOptions {
    feed: OrderFeed;
    /** Host restrictions combined with the feed's, rechecked before every write. */
    capabilities?: TradingCapabilitySource;
    constraints: OrderConstraints;
    mode?: TradeMode;
    /** Armed = fire immediately; otherwise the gate must approve each order. */
    armed?: boolean;
    gate?: GateFn;
    minModifyIntervalMs?: number;
    now?: () => number;
    idGen?: () => string;
    /** Called for modify validation and unsupported modify/cancel operations. */
    onValidationError?: (reason: string) => void;
    /**
     * How many settled orders stay readable before the oldest are dropped. A
     * trading session is a long-lived page and the per-order maps used to grow
     * for its whole life. 0 drops each order the moment it settles.
     */
    maxSettledOrders?: number;
    /** Host restrictions on the newer operations, combined with the feed's `features`; either can refuse. */
    features?: TradingFeatureSource;
    /**
     * The account selection. When set, every order and command is stamped with
     * it, one naming another account is refused, and a change while confirming
     * sends nothing. Pass the account view itself (an `AccountManager`) and a
     * selection from the other ledger is refused as well: a bare id cannot say
     * whether it is a live account about to take a sandbox order.
     */
    selectedAccount?: (() => string | null | undefined) | Pick<AccountStateSource, 'getState'>;
    /** Approves close, reverse and bracket commands when not armed. Omitted declines them. */
    confirmCommand?: (command: TradingCommand) => boolean | Promise<boolean>;
    /** Wall clock in UTC seconds, for expiry checks. Default `Date.now() / 1000`. */
    clock?: () => number;
}
interface PlaceResult {
    ok: boolean;
    clientId?: string;
    state?: ClientOrderState | undefined;
    /** Client-owned intent. `ok: true` means SUBMITTED, never acknowledged. */
    intent?: IntentState | undefined;
    reason?: string;
}
interface CommandResult extends PlaceResult {
    kind: TradingCommandKind;
    /** Client ids of a bracket's legs the provider has named, in its receipt or on its order stream. */
    legs?: {
        stopLoss?: string;
        takeProfit?: string;
    };
}

/**
 * Order engine (ARCHITECTURE.md §9.5): the chart-trading write path. Drives the
 * order state machine with: client-token idempotency, an arm/confirm gate,
 * pre-trade validation, rate-limited drag-modify, OCO linking, and analyzer
 * (sandbox) mode. Network-agnostic: it talks to an injected OrderFeed (the
 * FakeBroker simulates it in tests/demos).
 *
 * Three lifecycles are tracked per order and they are deliberately not merged:
 *
 *   state         the historical `ClientOrderState`. Existing consumers read it
 *                 and its meaning is unchanged, including the parts that are
 *                 optimistic (a resolved place() reads `working`).
 *   intent        what THIS client actually knows: submitted, ambiguous,
 *                 acknowledged, settled. A transport result never reaches past
 *                 SUBMITTED.
 *   brokerStatus  what the BROKER said. Undefined until an authoritative event
 *                 arrives (a stream or book update, or the broker's own explicit
 *                 refusal of the request), and written by nothing else.
 *
 * A resolved promise is not an order: it says the request left and an answer
 * came back, not that the exchange has anything. The v2 broker contract replaces
 * `state` outright with the two-field `OrderRow`; removing it here would break
 * every consumer of `ClientOrderState`, so the honest fields are added alongside
 * and the merge is deferred to v2.
 *
 * Closing, partly closing and reversing a position, and brackets whose legs the
 * provider links itself, are commands of their own with their own tokens. They
 * go only to a feed that declares and implements them. An opposite order is
 * never sent in place of a close: it can open a position the other way when
 * the one being closed has already gone, which is exactly the order nobody
 * asked for.
 */

declare class OrderEngine {
    private readonly _feed;
    private readonly _capabilities?;
    private readonly _features?;
    private readonly _selectedAccount?;
    private readonly _confirmCommand?;
    private readonly _clock;
    private readonly _constraints;
    private readonly _mode;
    private readonly _armed;
    private readonly _gate?;
    private readonly _minModifyMs;
    private readonly _now;
    private readonly _idGen;
    private readonly _onValidationError?;
    private readonly _maxSettled;
    private readonly _orders;
    private readonly _byBroker;
    private readonly _sentTokens;
    private readonly _lastModifyAt;
    private readonly _pendingModify;
    /** Settled client ids, oldest first: the eviction order for the maps above. */
    private readonly _settledIds;
    /** Leg rows that named an entry before it was bound, by entry broker id, then leg broker id. */
    private readonly _heldLegs;
    private _counter;
    constructor(opts: OrderEngineOptions);
    get mode(): TradeMode;
    state(clientId: string): ClientOrderState | undefined;
    /** Client-owned intent: what we did, as opposed to what the broker confirmed. */
    intentState(clientId: string): IntentState | undefined;
    /**
     * The broker's own last word. Undefined means the broker has said nothing
     * about this order yet, however far `state` has advanced on transport results.
     */
    brokerStatus(clientId: string): OrderStatus | undefined;
    /** Whether a row is a plain order, a position command, or a leg of a provider bracket. */
    orderKind(clientId: string): OrderKind | undefined;
    /** The account an order or command was sent for, fixed when it was sent. */
    orderAccount(clientId: string): string | undefined;
    /** Client ids of a provider bracket's legs. */
    bracketLegs(clientId: string): {
        stopLoss?: string;
        takeProfit?: string;
    } | undefined;
    private _capability;
    private _orderCapability;
    /** The feed must declare a feature; host features, when given, may refuse it as well. */
    private _feature;
    private _featureRequest;
    /**
     * Resolve the account an order goes to, writing it onto `req`. With a
     * selection configured the account is the one on screen; a request that
     * names a different one is refused rather than silently redirected.
     */
    private _account;
    /** The selected account, or why there is none this engine may stamp on an order. */
    private _selection;
    /** After an await: the account the user confirmed must still be the one selected. */
    private _accountStill;
    /** Duration, expiry and leverage: each is either carried by the provider or refused here. */
    private _schema;
    /** Place capability, then the newer request fields. Null when the order may go. */
    private _permitOrder;
    /** Price, trigger and quantity checks, with both prices snapped. */
    private _validate;
    /** The refusal for a token this engine has already put on the wire, if it has. */
    private _duplicate;
    /**
     * A command's confirmation, the same contract as the order gate: it runs
     * before any network call, so declining or throwing frees the token and the
     * same one may be offered again. Callers skip it when armed, so an armed
     * command reaches the feed in the same tick, as an armed order does.
     */
    private _confirmed;
    private _release;
    /**
     * The transport answered with an id. `state` keeps its historical optimism
     * for existing consumers; `intent` stops at SUBMITTED and `brokerStatus`
     * stays undefined, because the transport answering is not the exchange
     * answering. An order stream can describe the order before the transport
     * returns, and its word outranks the transport's, so a row the broker has
     * already described is left as the broker left it.
     */
    private _submitted;
    /** A write threw. Returns the reason to report. */
    private _failed;
    /**
     * A modify or cancel threw. It may still have been applied: only a
     * pre-flight failure rules that out, and leaves the order where it was last
     * known to be, unless it has moved on from `pending` since.
     */
    private _writeFailed;
    /** `ok` after a failure only when the broker's own stream already reports the write as accepted. */
    private _outcome;
    placeOrder(request: PlaceRequest): Promise<PlaceResult>;
    /**
     * Ask the provider what an order would cost. Read-only: it claims no token,
     * runs no confirmation and never calls `place`. The same checks as placing
     * apply, so a preview is never shown for an order that would be refused here.
     */
    previewOrder(request: PlaceRequest): Promise<PreviewResult>;
    /** Close a whole position, or part of it when `qty` is given, through the provider's own close. */
    closePosition(request: ClosePositionRequest): Promise<CommandResult>;
    /** Reverse a position through the provider's own reverse. */
    reversePosition(request: ReversePositionRequest): Promise<CommandResult>;
    /**
     * Whether a tracked command acts on the position a new one names. An
     * omitted exchange is whatever default the provider applies, so it matches
     * any exchange: keyed on the literal field, one position asked for with and
     * without its exchange let a second close out while the first was unresolved.
     */
    private static _samePosition;
    /**
     * Everything a close or reverse must pass, asked before confirmation and
     * again after it. The position lock is here: while an earlier close or
     * reverse on the same position has no known outcome, a second one could
     * reduce it twice or flip it back, so it waits for the broker's word.
     */
    private _permitCommand;
    private _positionCommand;
    /**
     * Place an entry whose stop and target legs the provider places and links
     * itself. The legs are never linked client-side: one-cancels-other is the
     * provider's job here, and a second, client-side link would race it.
     */
    placeBracket(request: BracketOrderRequest): Promise<CommandResult>;
    /**
     * The row for one leg of a provider bracket, made on first sight. The
     * receipt names the legs when it arrives; the order stream or a read of the
     * book can name them first, or instead, when the receipt was lost.
     */
    private _leg;
    /** One-click market order. Omitting `opts` is exactly the previous behaviour. */
    placeMarket(symbol: string, side: OrderSide, qty: number, opts?: MarketOrderOptions): Promise<PlaceResult>;
    /** Link two orders as OCO: when one fills/cancels, the other is cancelled. */
    linkOco(clientIdA: string, clientIdB: string): void;
    /**
     * Rate-limited modify (drag): coalesces to the latest, sends at most every
     * minModifyMs. An invalid price (tick/band/freeze) is NOT enqueued or sent:
     * it surfaces via `onValidationError` so the UI can snap the line back.
     */
    requestModify(clientId: string, price: number, opts?: ModifyOptions): void;
    /** Force-send any pending modify (e.g. on drag end). */
    commitModify(clientId: string): Promise<void>;
    /**
     * Turn a dragged level into a patch the broker can apply without losing a
     * field it already holds.
     */
    private _buildModifyPatch;
    private _validateTrigger;
    private _flushModify;
    cancelOrder(clientId: string): Promise<void>;
    /** Broker fill event (by broker id). Advances state and triggers OCO. */
    onFill(brokerId: string, full: boolean): void;
    /**
     * Authoritative state from the broker: an order-stream push or a poll of the
     * book. This is the ONLY input that writes `brokerStatus`; place, modify and
     * cancel resolving never do.
     */
    onBrokerUpdate(brokerId: string, status: OrderStatus): void;
    /**
     * An order row from the broker's stream or book, matched by broker id or,
     * failing that, by the client token the broker echoes. The token is the only
     * handle on a write whose answer was lost: it is how an AMBIGUOUS row learns
     * its broker id and its outcome from the broker rather than from a guess.
     * Rows may come in any order: a bracket leg that names an entry not yet
     * bound is held, and applied when the entry binds.
     */
    onBrokerOrder(update: BrokerOrderUpdate): void;
    /**
     * Give a row its broker id. A bracket entry then takes the leg rows that
     * named it before it was bound, ahead of its own status: a filled entry
     * can settle and leave the ring at once, and its legs would have no entry
     * left to find.
     */
    private _bind;
    /**
     * Keep a leg row whose entry has no broker id here yet, until `_bind`
     * gives it one. A provider that echoes no leg token names a leg only by
     * its entry, and may well report the leg first: its history lists the
     * newest rows first, and its stream can describe the legs before the
     * transport returns the entry's id. Rows are held only while a bracket
     * this engine sent is still waiting for that id, since no other could
     * ever claim them, and for a bounded number of entries, since other
     * sessions' brackets arrive alongside and are never claimed. The latest
     * row for a leg is the one kept.
     */
    private _hold;
    /** Whether a bracket this engine sent could still be bound to an entry the broker reports. */
    private _awaitingEntry;
    /**
     * The row a broker id not seen before belongs to: the one its echoed token
     * names, or a leg of a bracket this engine sent, found by the leg's own
     * token or by its entry and role. Adopting a leg on first sight is how a
     * bracket whose answer was lost still gets legs that can be cancelled,
     * modified and filled. A leg found only by an entry not yet bound is held
     * until the entry binds, so the order the broker reports rows in does not
     * matter.
     */
    private _unbound;
    /**
     * Drop an AMBIGUOUS row and free its token, after the host has established
     * from the broker's complete book that the request never arrived. Nothing
     * here can decide that on its own, which is why it is a separate call. Any
     * other row is left alone and the call returns false.
     */
    releaseAmbiguous(clientId: string): boolean;
    /** A close or reverse is not an order the user can drag or cancel from here. */
    private _isPositionCommand;
    private _cancelOcoPeer;
    /**
     * Enter reconciliation: the connection dropped or a gap was seen, so our
     * picture may be behind. Nothing is concluded here; call `onReconnect` with
     * the fresh book to conclude anything. An AMBIGUOUS row is left alone, since
     * only the broker can take it out of that state.
     */
    beginReconcile(): void;
    /**
     * Apply a fresh order-book snapshot. Absence is not death: `state` still goes
     * to `stale` for existing consumers, but the intent becomes AMBIGUOUS, because
     * an order missing from one snapshot means our picture is incomplete, not that
     * the broker cancelled it. Such a row is never pruned. (v2 drops `stale`
     * entirely; that removal has to wait for the contract bump.)
     */
    onReconnect(presentBrokerIds: ReadonlySet<string>): void;
    /**
     * Terminal bookkeeping. Per-order scratch (throttle stamps, coalesced patches)
     * goes at once; the row itself survives so a host can still read the final
     * state, and only the oldest are evicted once `maxSettledOrders` is passed.
     *
     * Idempotency tokens are never pruned. They are the record of what this client
     * has put on the wire, and dropping one so a map stays small is the same
     * mistake as releasing it on a transport error.
     */
    private _settle;
}

/**
 * Deterministic in-memory broker simulator (ARCHITECTURE.md §11.1). Holds order
 * and position snapshots and notifies subscribers, so the trade layer can be
 * tested and demoed with zero network. It answers the order engine's place,
 * modify and cancel with an order state machine of its own, and a host that
 * only reads can seed snapshots and emit LTP.
 *
 * Constructed with `accounts`, it also simulates a provider with account
 * ledgers: balances and margin, fills at a mark price, executions and order
 * history, preview, durations, leverage, native close and reverse, and
 * brackets whose legs it links itself. Without `accounts` it declares none of
 * that and behaves exactly as it always has, so the engine refuses the newer
 * operations against it instead of approximating them.
 *
 * It refuses the way a server would, with an error marked `rejected`, when a
 * request names an unknown account, the other ledger, a feature it was told not
 * to support, or margin it does not have. Test hooks can hold, fail or lose the
 * answer to any call, and drop the connection, so the client's handling of each
 * is testable against the same ledger the reference host uses.
 *
 * A dropped connection fails each call the way a client's own transport would:
 * one made while it is down never leaves, and says so with a pre-flight marker,
 * so the engine blocks it and frees its token. One already out when it drops
 * is not applied and fails with a plain error, which is all a client would see
 * of a lost answer too; only the broker's book can settle that one.
 */

/** One simulated account. */
interface FakeAccountSeed {
    id: string;
    name?: string;
    mode: TradeMode;
    currency?: string;
    /** Opening cash balance. */
    balance: number;
    /** Margin multiplier when an order names none. Default 1. */
    leverage?: number;
    /** Highest leverage an order may request. Default: `leverage`. */
    maxLeverage?: number;
}
/** Each call a test can hold with `latency` or fail with `failNext`. */
type FakeBrokerOperation = 'place' | 'modify' | 'cancel' | 'preview' | 'close' | 'reverse' | 'bracket' | 'accounts' | 'snapshot' | 'positions' | 'executions' | 'history';
/**
 * `reject`: the server refuses (authoritative). `timeout`: the request is lost
 * before it is applied. `lost-response`: it is applied and the answer is lost.
 * The client cannot tell the last two apart, which is the point.
 */
type FakeBrokerFailure = 'reject' | 'timeout' | 'lost-response';
interface FakeBrokerOptions {
    /** Enables account ledgers. Omitted keeps the original book-only simulation. */
    accounts?: readonly FakeAccountSeed[];
    /** What the simulated provider declares. Default with accounts: every feature and duration. */
    features?: TradingFeatureSource;
    /** Clock in UTC seconds for snapshots, executions, history and expiry. Default `Date.now() / 1000`. */
    now?: () => number;
    /** Awaited before each call is answered, so a test can hold one account's answer and release it later. */
    latency?: (operation: FakeBrokerOperation, accountId: string | undefined) => void | Promise<void>;
}
/** Who an order belongs to and what it was, beside the order itself. */
interface FakeOrderInfo {
    accountId?: string;
    clientToken?: string;
    command?: 'close' | 'reverse' | 'bracket';
}
declare class FakeBroker implements OrderFeed, AccountFeed {
    private _orders;
    private _positions;
    private readonly _bookListeners;
    private readonly _ltpListeners;
    private readonly _depthListeners;
    private readonly _orderListeners;
    private _idCounter;
    private _execCounter;
    /**
     * Test hook: set to a reason to make the next `place()` throw it once, as a
     * plain error. That is an answer lost on its way back, not a refusal: the
     * engine settles the order as ambiguous and keeps its token, since it may
     * have reached the broker. For a broker's refusal, construct with
     * `accounts` and call `failNext('place', 'reject', reason)`.
     */
    rejectNextPlace: string | null;
    private readonly _ledgers;
    private readonly _meta;
    private readonly _marks;
    private readonly _accountMode;
    private readonly _features?;
    private readonly _clock;
    private readonly _latency?;
    private _failures;
    private _connected;
    private _muted;
    constructor(options?: FakeBrokerOptions);
    /** What this simulated provider declares. Undefined without accounts. */
    get features(): TradingFeatureSource | undefined;
    onBook(cb: (orders: Order[], positions: Position[]) => void): void;
    onLtp(cb: (symbol: string, ltp: number) => void): void;
    /** The order stream: every status change of every order, with the account and client token behind it. */
    onOrderUpdate(cb: (order: Order, info: FakeOrderInfo) => void): void;
    /** Replace the current book snapshot and notify (simulates a poll / WS update / reconnect). */
    setBook(orders: Order[], positions: Position[]): void;
    emitLtp(symbol: string, ltp: number): void;
    /** The price market orders fill at and positions are valued at. Pushes fresh figures to holders. */
    setMark(symbol: string, price: number): void;
    onDepth(cb: (symbol: string, depth: MarketDepth) => void): void;
    emitDepth(symbol: string, depth: MarketDepth): void;
    /** Build a deterministic N-level synthetic book around `ltp` (demo/test helper). */
    static makeDepth(ltp: number, levels: number, tickSize?: number): MarketDepth;
    orders(): readonly Order[];
    positions(): readonly Position[];
    /** An account's open positions, synchronously (test and demo helper). */
    accountPositions(accountId: string): Position[];
    /** Make the next call of `operation` fail as described (test hook). */
    failNext(operation: FakeBrokerOperation, failure: FakeBrokerFailure, reason?: string): void;
    /**
     * Drop the connection. Every call fails without applying: one made while it
     * is down as never sent (`isPreflightFailure`), one already out with a plain
     * error. Account streams report the loss.
     */
    disconnect(): void;
    reconnect(): void;
    /** Stop delivering order-stream updates, as a dropped stream would (test hook). */
    muteOrderUpdates(muted: boolean): void;
    place(req: PlaceRequest & {
        mode: TradeMode;
    }): Promise<{
        orderId: string;
    }>;
    modify(orderId: string, patch: {
        price?: number;
        triggerPrice?: number;
        qty?: number;
    }): Promise<void>;
    cancel(orderId: string): Promise<void>;
    /** Simulate a fill for an order (test hook). */
    fill(orderId: string): void;
    previewOrder(req: PlaceRequest & {
        mode: TradeMode;
    }): Promise<OrderPreview>;
    closePosition(req: ClosePositionRequest & {
        mode: TradeMode;
    }): Promise<CommandReceipt>;
    reversePosition(req: ReversePositionRequest & {
        mode: TradeMode;
    }): Promise<CommandReceipt>;
    placeBracket(req: BracketOrderRequest & {
        mode: TradeMode;
        legClientTokens?: {
            stopLoss: string;
            takeProfit: string;
        };
    }): Promise<BracketReceipt>;
    listAccounts(_signal?: AbortSignal): Promise<readonly TradingAccount[]>;
    getAccountSnapshot(accountId: string, _signal?: AbortSignal): Promise<AccountSnapshot>;
    subscribeAccount(accountId: string, onSnapshot: (s: AccountSnapshot) => void, onError?: (e: unknown) => void): () => void;
    getAccountPositions(accountId: string, _signal?: AbortSignal): Promise<readonly Position[]>;
    getExecutions(query: AccountHistoryQuery, _signal?: AbortSignal): Promise<readonly Execution[]>;
    getOrderHistory(query: AccountHistoryQuery, _signal?: AbortSignal): Promise<readonly OrderHistoryEntry[]>;
    private _notify;
    private _emit;
    /** Book listeners and the account's own stream both learn of a ledger change. */
    private _changed;
    private _push;
    /** A call made while the connection is down never leaves the client. */
    private _online;
    /** Latency, then the connection, then any queued failure. True: apply, then lose the answer. */
    private _enter;
    /**
     * A server refuses to change an order that is already over. Market and
     * command orders stay in the book as filled, so without this a late cancel
     * rewrote a fill as cancelled while its execution and position stayed.
     */
    private _live;
    private _lost;
    private _require;
    private _account;
    /** The named account, or the first of the order's mode; either must belong to that mode. */
    private _ledgerFor;
    private _mark;
    private _holding;
    /** What an order would trade at: the mark for a market order, else its own level. */
    private _entryPrice;
    /** The part of an order that adds exposure; reducing an open position needs no new margin. */
    private _opening;
    /** Durations, expiry and leverage as this provider accepts them. Throws a refusal. */
    private _schema;
    /** Accept an order into an account: checks, margin, then fill, rest or cancel by type and duration. */
    private _placeIn;
    /** A native position command, booked as one order the provider marks as that command. */
    private _command;
    /** Book a fill into the account: average in, realize out, flip at the fill price. */
    private _execute;
    private _snapshot;
    /** Record a final state for an order already out of the book. */
    private _close;
    /** Lapse GTD orders whose expiry has passed. */
    private _sweep;
    private _window;
}

/** Use the adapter's quantity grid directly; this function never converts units or places orders. */
declare function orderConstraintsForInstrument(instrument: Instrument): OrderConstraints;

/**
 * Depth-of-market ladder (ARCHITECTURE.md §9.4). Docked to the right of the
 * plot, price-aligned to the price scale. Depth-agnostic: it reads the level
 * count from the payload (5 / 20 / 30 / 50 / 200) at runtime. Viewport
 * virtualization keeps a deep book at 60 fps; price-bucket aggregation compacts
 * deep books; a size heatmap highlights resting liquidity; and it degrades
 * gracefully to nothing when no depth is available.
 *
 * Pure helpers (capability / aggregation / virtualization) are split out for
 * unit testing; the primitive draws on the top (overlay) canvas so frequent
 * depth updates only repaint the cheap overlay.
 */

type LadderTier = 'none' | 'compact' | 'deep';
/** Capability tier from the live payload: drives graceful degradation. */
declare function ladderCapability(depth: MarketDepth): LadderTier;
interface LadderRow {
    price: number;
    bidQty: number;
    askQty: number;
}
/**
 * Merge bids + asks into price rows, optionally bucketing every `groupBy` ticks
 * (price-step aggregation for deep books). Returns rows sorted high to low price.
 * Pass a `TickSchedule` in place of the tick size for an instrument whose tick
 * changes with price: each row is then a price its band allows, and a group
 * spans `groupBy` ticks of that band, so the step changes at a boundary.
 * There `groupBy` counts whole ticks, a fraction rounding down (2.5 groups by
 * 2), because a fractional group would label rows between the prices a band
 * allows. The constant path multiplies `tickSize * groupBy` as it always has.
 */
declare function buildRows(depth: MarketDepth, tickSize: number | TickSchedule, groupBy?: number): LadderRow[];
/**
 * Virtualize: keep only rows whose y is inside the plot (± one row) and cap the
 * count to `maxRows` nearest the vertical centre (around the LTP). This is what
 * keeps a 200-level book cheap and readable.
 */
declare function visibleRows(rows: readonly LadderRow[], priceToY: (p: number) => number, plotHeight: number, rowHeight: number, maxRows: number): LadderRow[];
interface DomLadderOptions {
    tickSize: number;
    /**
     * Price-dependent ticks. When set, every row is a price the instrument can
     * trade at in its own band, `groupBy` counts whole ticks of that band (a
     * fraction rounds down), and `tickSize` is unused. Absent or null, rows step
     * by `tickSize`.
     */
    tickSchedule?: TickSchedule | null;
    /** Strip width in media px. */
    width: number;
    /**
     * Group every N ticks into one row (deep-book aggregation). With a
     * `tickSchedule`, N is whole ticks of each band: 2.5 groups by 2.
     */
    groupBy: number;
    /** Max rows drawn per frame (virtualization cap). */
    maxRows: number;
    rowHeight: number;
}
declare const DEFAULT_DOM_LADDER_OPTIONS: DomLadderOptions;
declare class DomLadder implements IPrimitive {
    private _opts;
    private _depth;
    private _host;
    private _rowHits;
    constructor(options?: Partial<DomLadderOptions>);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    setDepth(depth: MarketDepth): void;
    tier(): LadderTier;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
}

declare const TRADE_TIER: "trade";

export { type AccountFeed, type AccountHistoryQuery, AccountManager, type AccountManagerOptions, type AccountReadResult, type AccountSelectResult, type AccountSnapshot, type AccountState, type AccountStateSource, type AccountStatus, BracketGroup, type BracketOrderRequest, type BracketReceipt, type BracketState, type BrokerOrderUpdate, type BrokerRejection, type ClientOrderState, type ClosePositionRequest, type CommandReceipt, type CommandResult, DEFAULT_DOM_LADDER_OPTIONS, DomLadder, type DomLadderOptions, type Execution, type FakeAccountSeed, FakeBroker, type FakeBrokerFailure, type FakeBrokerOperation, type FakeBrokerOptions, type FakeOrderInfo, type GateFn, type IntentState, type LadderRow, type LadderTier, type MarketOrderOptions, type ModifyOptions, type ModifyPatch, ORDER_DURATIONS, type Order, type OrderConstraints, type OrderDuration, OrderEngine, type OrderEngineOptions, type OrderEvent, type OrderFeed, type OrderHistoryEntry, type OrderKind, type OrderPreview, type OrderRole, type OrderSide, type OrderStatus, type OrderType, type PlaceRequest, type PlaceResult, type Position, type PositionCommandRequest, PositionMarker, type PreflightFailure, type PreviewResult, type PriceBand, type ReversePositionRequest, TRADE_TIER, TradeController, type TradeHost, type TradeMode, type TradingAccount, type TradingCommand, type TradingCommandKind, type TradingFeature, type TradingFeatureRequest, type TradingFeatureSource, type TradingFeatures, type ValidationCode, type ValidationResult, WorkingOrderLine, bracketValid, breakeven, buildRows, canTransition, checkTradingFeature, isBrokerRejection, isPreflightFailure, isTerminal, isWorking, ladderCapability, orderConstraintsForInstrument, riskReward, tradingFeatureLabel, transition, unrealizedPnl, unrealizedPnlPercent, validateOrder, validatePrice, validateQuantity, visibleRows, withinPriceBand };
