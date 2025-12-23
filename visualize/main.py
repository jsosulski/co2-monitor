import pandas as pd
from dash import Dash, dcc, html
from dash.dependencies import Input, Output

from plotly.subplots import make_subplots
import plotly.graph_objects as go


CSV_PATH = "../log.csv"

YELLOW_LIMIT = 800
RED_LIMIT = 1200
DEFAULT_MAX = 1600

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

    df["co2_for_plot"] = df["co2_ppm"].where(df["co2_is_valid"], None)

    fig = make_subplots(specs=[[{"secondary_y": True}]])

    co2_min = min(400, df["co2_for_plot"].min()) - 10
    co2_max = max(DEFAULT_MAX, df["co2_for_plot"].max()) + 10
    temp_min = min(15, df["temperature"].min()) - 0.5
    temp_max = max(25, df["temperature"].max()) + 0.5

    fig.add_shape(
        type="rect",
        xref="paper", x0=0, x1=1,
        yref="y2", y0=YELLOW_LIMIT, y1=RED_LIMIT,
        fillcolor="yellow",
        opacity=0.17,
        layer="below",
        line_width=0
    )

    fig.add_shape(
        type="rect",
        xref="paper", x0=0, x1=1,
        yref="y2", y0=RED_LIMIT, y1=co2_max,
        fillcolor="red",
        opacity=0.12,
        layer="below",
        line_width=0
    )


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
          
    fig.update_yaxes(
        range=[temp_min, temp_max],
        title_text="Temperature (°C)",
        secondary_y=False
    )

    fig.update_yaxes(
        range=[co2_min, co2_max],
        title_text="CO₂ (ppm)",
        secondary_y=True
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
                text="CO₂ (ppm)",
                font=dict(color="blue")
            ),
            tickfont=dict(color="blue")
        )
    )

    return fig

if __name__ == "__main__":
    app.run(debug=False, host="0.0.0.0")
