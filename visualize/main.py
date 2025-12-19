import pandas as pd
from dash import Dash, dcc, html
from dash.dependencies import Input, Output

from plotly.subplots import make_subplots
import plotly.graph_objects as go


CSV_PATH = "../log.csv"

app = Dash(__name__)

app.layout = html.Div([
    html.H3("Live CO₂ & Temperature Monitor"),
    dcc.Graph(id='live-graph'),
    dcc.Interval(id='interval', interval=15*1000, n_intervals=0)
])

@app.callback(
    Output('live-graph', 'figure'),
    Input('interval', 'n_intervals')
)
def update_graph(n):
    df = pd.read_csv(CSV_PATH, parse_dates=['timestamp'])

    # --- Prepare CO₂ data with gaps for invalid values ---
    df["co2_for_plot"] = df["co2_ppm"].where(df["co2_is_valid"], None)

    fig = make_subplots(specs=[[{"secondary_y": True}]])

    # --- CO₂ background bands ---
    co2_max = max(2000, df["co2_ppm"].max())

    # --- CO₂ background bands (explicit yref=y2) ---
    co2_max = max(2000, df["co2_ppm"].max())

    fig.add_shape(
        type="rect",
        xref="paper", x0=0, x1=1,
        yref="y2", y0=1000, y1=1500,
        fillcolor="yellow",
        opacity=0.17,
        layer="below",
        line_width=0
    )

    fig.add_shape(
        type="rect",
        xref="paper", x0=0, x1=1,
        yref="y2", y0=1500, y1=co2_max,
        fillcolor="red",
        opacity=0.12,
        layer="below",
        line_width=0
    )


    # --- Plot CO₂ ---
    fig.add_trace(
        go.Scatter(
            x=df['timestamp'],
            y=df['co2_for_plot'],
            mode='lines',
            name='CO₂ (ppm)',
            line=dict(color="blue"),
            connectgaps=False
        ),
        secondary_y=True
    )

    # --- Plot Temperature ---
    fig.add_trace(
        go.Scatter(
            x=df['timestamp'],
            y=df['temperature'],
            mode='lines',
            name='Temperature (°C)',
            line=dict(color="red")
        ),
        secondary_y=False
    )

    # --- Add shaded regions for invalid CO₂ ---
    invalid = df[df['co2_is_valid'] == False]

    if not invalid.empty:
        invalid["gap"] = (invalid["timestamp"].diff() > pd.Timedelta("1s")).cumsum()

        for _, group in invalid.groupby("gap"):
            start = group["timestamp"].iloc[0]
            end   = group["timestamp"].iloc[-1]

            fig.add_vrect(
                x0=start, x1=end,
                fillcolor="red",
                opacity=0.15,
                line_width=0
            )

    fig.update_layout(
        title="CO₂ and Temperature Over Time",
        legend=dict(x=0, y=1.1, orientation="h"),
        template="plotly_white"
    )

    fig.update_yaxes(showgrid=False, secondary_y=False)
    fig.update_layout(
        yaxis=dict(
            title=dict(
                text="Temperature (°C)",
                font=dict(color="red")
            ),
            tickfont=dict(color="red")
        ),
        yaxis2=dict(
            title=dict(
                text="CO2 (ppm)",
                font=dict(color="blue")
            ),
            tickfont=dict(color="blue")
        )
    )

    return fig

if __name__ == "__main__":
    app.run(debug=False, host="0.0.0.0")
