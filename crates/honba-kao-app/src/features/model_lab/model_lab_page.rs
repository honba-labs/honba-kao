use leptos::prelude::*;

/// Model Lab page — experiment registry and leaderboard.
#[component]
pub fn ModelLabPageContent() -> impl IntoView {
    view! {
        <section class="prose max-w-5xl">
            <h2 class="text-xl font-semibold mb-4">Experiment Registry</h2>
            <p class="text-muted-foreground mb-4">
                "List of versioned backtest runs with key metrics."
            </p>
            <div class="overflow-x-auto">
                <table class="min-w-full rounded-md border border-border-default">
                    <thead>
                        <tr>
                            <th class="left text-left p-3 font-medium">Run ID</th>
                            <th class="left text-left p-3 font-medium">Sharpe</th>
                            <th class="left text-left p-3 font-medium">CAGR</th>
                            <th class="left text-left p-3 font-medium">Max DD</th>
                            <th class="left text-left p-3 font-medium">Status</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td class="p-3">run/20240115-01</td>
                            <td class="p-3">1.42</td>
                            <td class="p-3">28.5%</td>
                            <td class="p-3">-15.3%</td>
                            <td class="p-3"><span class="px-2 py-1 rounded-xs bg-surface-panel">Paper</span></td>
                        </tr>
                        <tr>
                            <td class="p-3">run/20240222-03</td>
                            <td class="p-3">1.28</td>
                            <td class="p-3">24.1%</td>
                            <td class="p-3">-18.7%</td>
                            <td class="p-3"><span class="px-2 py-1 rounded-xs bg-surface-panel">Live</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">Model Comparison</h2>
            <p class="text-muted-foreground mb-4">
                "Side-by-side metric overlay for up to 4 models."
            </p>
        </section>
    }
}
