from datetime import timedelta, datetime
import pandas as pd
from dash import Dash, dcc, html, no_update
from dash.dependencies import Input, Output

from plotly.subplots import make_subplots
import plotly.graph_objects as go

import lttb
import numpy as np


CSV_PATH = "../log.csv"

YELLOW_LIMIT = 800
RED_LIMIT = 1200
DEFAULT_MAX = 1600

app = Dash(__name__)

app.layout = html.Div([
    html.H3("Live CO₂ & Temperature Monitor"),
    html.Div(id="current"),
    dcc.Checklist(id="options-checklist", options=[{"label": "Auto update", "value": "auto-update"}, {"label": "Naive sampling", "value": "naive"}, {"label": "Fixed y-lim", "value": "fixed_ylim"}], value=["auto-update", "fixed_ylim"], inline=True),
    dcc.RadioItems(
        id="interval-select", options=[
            {"label": "Full", "value": "full"},
            {"label": "3 days", "value": "3d"},
            {"label": "1 day", "value": "1d"},
            {"label": "1 hour", "value": "1h"},
        ], value="full", inline=True,
    ),
    dcc.Graph(id='live-graph', config={"displayModeBar": False}),
    dcc.Interval(id='interval', interval=10*1000, n_intervals=0)
])

@app.callback(
    Output('live-graph', 'figure'),
    Output('current', 'children'),
    Input('interval', 'n_intervals'),
    Input('options-checklist', "value"),
    Input('interval-select', "value")
)
def update_graph(n, options, interval):
    options = options or []
    use_naive = "naive" in options
    auto_update = "auto-update" in options
    use_lttb = not use_naive
    if not auto_update:
        return no_update, no_update
    df = pd.read_csv(CSV_PATH, parse_dates=['timestamp'])

    df["co2_for_plot"] = df["co2_ppm"]#.where(df["co2_is_valid"], None)

    now = datetime.now()

    if interval == "3d":
        df = df.loc[df["timestamp"] >= now - timedelta(hours=72)]
    if interval == "1d":
        df = df.loc[df["timestamp"] >= now - timedelta(hours=24)]
    elif interval == "1h":
        df = df.loc[df["timestamp"] >= now - timedelta(hours=1)]

    fig = make_subplots(specs=[[{"secondary_y": True}]])

    co2_now = df["co2_for_plot"].iloc[-1]
    co2_min = min(400, df["co2_for_plot"].min()) - 10
    co2_max = max(DEFAULT_MAX, df["co2_for_plot"].max()) + 10
    temp_now = df["temperature"].iloc[-1]
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

    n_out = 1000

    if len(df) <= n_out:
        co2_df = df[["timestamp", "co2_for_plot"]].copy()
        temp_df = df[["timestamp", "temperature"]].copy()
    else:
        if use_lttb:
            co2_df = lttb.downsample(
                np.column_stack([df["timestamp"].astype("int64"), df["co2_for_plot"]]),
                n_out=n_out
            )
            co2_df = pd.DataFrame(
                co2_df,
                columns=["timestamp", "co2_for_plot"]
            )
            co2_df["timestamp"] = pd.to_datetime(co2_df["timestamp"])
            temp_df = lttb.downsample(
                np.column_stack([df["timestamp"].astype("int64"), df["temperature"]]),
                n_out=n_out
            )
            temp_df = pd.DataFrame(
                temp_df,
                columns=["timestamp", "temperature"]
            )
            temp_df["timestamp"] = pd.to_datetime(temp_df["timestamp"])
        else:
            idx = np.linspace(0, len(df) - 1, n_out, dtype=int)
            co2_df = df.iloc[idx][["timestamp", "co2_for_plot"]].copy()
            temp_df = df.iloc[idx][["timestamp", "temperature"]].copy()


    fig.add_trace(
        go.Scatter(
            x=co2_df['timestamp'],
            y=co2_df['co2_for_plot'],
            name='CO₂ (ppm)',
            mode='lines+markers',
            line=dict(color="blue", dash="dot"),
            connectgaps=False
        ),
        secondary_y=True
    )

    fig.add_trace(
        go.Scatter(
            x=temp_df['timestamp'],
            y=temp_df['temperature'],
            mode='lines+markers',
            line=dict(color="red", dash="dot"),
            name='Temperature (°C)',
        ),
        secondary_y=False
    )

    invalid = df[df['co2_is_valid'] == False]

    # if not invalid.empty:
    #     invalid["gap"] = (invalid["timestamp"].diff() > pd.Timedelta("1s")).cumsum()

    #     for _, group in invalid.groupby("gap"):
    #         start = group["timestamp"].iloc[0]
    #         end   = group["timestamp"].iloc[-1]

    #         fig.add_vrect(
    #             x0=start, x1=end,
    #             fillcolor="red",
    #             opacity=0.15,
    #             line_width=0
    #         )

    fixed_ylim = "fixed_ylim" in options

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
          
    fig.update_yaxes(
        range=[temp_min, temp_max] if fixed_ylim else [None, None],
        title_text="Temperature (°C)",
        secondary_y=False
    )

    fig.update_yaxes(
        range=[co2_min, co2_max] if fixed_ylim else [None, None],
        title_text="CO₂ (ppm)",
        secondary_y=True
    )

    current = f"Aktuell: {co2_now} CO₂ (ppm) und {temp_now:.1f}°C"

    return fig, current

if __name__ == "__main__":
    app.run(debug=False, host="0.0.0.0", port="80")
