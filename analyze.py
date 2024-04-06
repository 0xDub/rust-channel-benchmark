import matplotlib.pyplot as plt
import pandas as pd

plt.style.use("dark_background")


class bcolors:
    HEADER = "\033[95m"
    OKBLUE = "\033[94m"
    OKCYAN = "\033[96m"
    OKGREEN = "\033[92m"
    WARNING = "\033[93m"
    FAIL = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    UNDERLINE = "\033[4m"


# =------------------------------------------------------------= #
def get_available_colors():
    return [
        "#21fced",
        "#21fc80",
        "#fc2130",
        "#219efc",
        "#ed21fc",
        "#8021fc",
        "#30fc21",
        "#fc8021",
        "#FFFF00",
        "#FDECA0",
        "#FF00FF",
        "#00FFFF",
        "#FF0000",
        "#00FF00",
        "#0000FF",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
        "#FF80FF",
        "#80FFFF",
        "#FF80FF",
        "#FFFF80",
        "#FF8000",
        "#FF0080",
        "#80FF00",
        "#80FF00",
        "#0080FF",
        "#8000FF",
        "#FF8080",
        "#80FF80",
        "#8080FF",
    ]


def analyze_inter_async(bins, sender_threshold, receiver_threshold, show_graph=False):
    available_colors = get_available_colors()
    methods = [
        "flume",
        "kanal",
        "tokio",
        "async_channel",
        "crossfire",
        "thingbuf",
        "tachyonix",
        "postage",
        "async_broadcast",
    ]  #

    aggregate_data = pd.DataFrame(columns=["latency", "method"])

    print()
    print(
        bcolors.OKCYAN
        + "=-= Inter-Process (Async) | Latency Stats (ns) =-="
        + bcolors.ENDC
    )

    fig, sender_ax = plt.subplots(1)
    sender_ax.set_title("Inter-Process (Async) | Sender | Latency Distribution")
    sender_ax.set_xlabel("Latency (ns)")
    sender_ax.set_ylabel("Density")

    fig, receiver_ax = plt.subplots(1)
    receiver_ax.set_title("Inter-Process (Async) | Receiver | Latency Distribution")
    receiver_ax.set_xlabel("Latency (ns)")
    receiver_ax.set_ylabel("Density")

    for method in methods:
        # =------------------------------------------------------------= #
        with open(f"inter_latencies/async/{method}_sender.txt", "r") as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        sender_data = pd.DataFrame(lines, columns=["latency"])
        sender_data = sender_data.astype(int)
        sender_data = sender_data[sender_data["latency"] < sender_threshold]
        sender_data["method"] = method
        sender_data["direction"] = "sender"
        if len(aggregate_data) == 0:
            aggregate_data = sender_data
        else:
            aggregate_data = pd.concat([aggregate_data, sender_data])
        # =------------------------------------------------------------= #
        with open(f"inter_latencies/async/{method}_receiver.txt", "r") as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        recv_data = pd.DataFrame(lines, columns=["latency"])
        recv_data = recv_data.astype(int)
        recv_data = recv_data[recv_data["latency"] < receiver_threshold]
        recv_data["method"] = method
        recv_data["direction"] = "receiver"
        if len(aggregate_data) == 0:
            aggregate_data = recv_data
        else:
            aggregate_data = pd.concat([aggregate_data, recv_data])
        # =------------------------------------------------------------= #

        alpha = 0.25

        # get color from available colors
        color = available_colors.pop(0)
        sender_ax.hist(
            sender_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        sender_ax.axvline(
            sender_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )
        receiver_ax.hist(
            recv_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        receiver_ax.axvline(
            recv_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )

    for direction in ["sender", "receiver"]:
        print()
        print(
            bcolors.OKGREEN
            + f"=-------------= {direction.capitalize()} | Latencies =-------------="
            + bcolors.ENDC
        )
        pre_sorted_data = aggregate_data[(aggregate_data["direction"] == direction)]
        # group by method, calculate mean of latency, then sort by mean latency
        pre_sorted_data = (
            pre_sorted_data.groupby("method")
            .agg({"latency": ["mean", "median", "std"]})
            .reset_index()
        )
        sorted_by_mean = pre_sorted_data.sort_values(by=[("latency", "mean")])
        for _, row in sorted_by_mean.iterrows():
            method = row["method"].iloc[0]
            mean_latency = format(row[("latency", "mean")] / 1000, ".2f")
            median_latency = format(row[("latency", "median")] / 1000, ".2f")
            std_latency = format(row[("latency", "std")] / 1000, ".2f")
            # format the lines to be more readable
            padding = " " * (20 - len(method))
            print(
                f"{method}{padding}| mu: {mean_latency}us | median: {median_latency}us | std: {std_latency}us"
            )

    sender_ax.legend()
    receiver_ax.legend()

    if show_graph:
        plt.show()


def analyze_inter_busy(bins, sender_threshold, receiver_threshold, show_graph=False):

    available_colors = get_available_colors()
    methods = [
        "rtrb",
        "ringbuf",
        "crossbeam_channel",
        "crossbeam_queue",
        "kanal",
        "flume",
        "omango",
        "npnc",
        "magnetic",
        "tokio",
    ]  #

    aggregate_data = pd.DataFrame(columns=["latency", "method"])

    print()
    print(
        bcolors.OKCYAN
        + "=-= Inter-Process (Busy) | Latency Stats (ns) =-="
        + bcolors.ENDC
    )

    fig, sender_ax = plt.subplots(1)
    sender_ax.set_title("Inter-Process (Busy) | Sender | Latency Distribution")
    sender_ax.set_xlabel("Latency (ns)")
    sender_ax.set_ylabel("Density")

    fig, receiver_ax = plt.subplots(1)
    receiver_ax.set_title("Inter-Process (Busy) | Receiver | Latency Distribution")
    receiver_ax.set_xlabel("Latency (ns)")
    receiver_ax.set_ylabel("Density")

    for method in methods:
        # =------------------------------------------------------------= #
        with open(f"inter_latencies/busy/{method}_sender.txt", "r") as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        sender_data = pd.DataFrame(lines, columns=["latency"])
        sender_data = sender_data.astype(int)
        sender_data = sender_data[sender_data["latency"] < sender_threshold]
        sender_data["method"] = method
        sender_data["direction"] = "sender"
        if len(aggregate_data) == 0:
            aggregate_data = sender_data
        else:
            aggregate_data = pd.concat([aggregate_data, sender_data])
        # =------------------------------------------------------------= #
        with open(f"inter_latencies/busy/{method}_receiver.txt", "r") as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        recv_data = pd.DataFrame(lines, columns=["latency"])
        recv_data = recv_data.astype(int)
        recv_data = recv_data[recv_data["latency"] < receiver_threshold]
        recv_data["method"] = method
        recv_data["direction"] = "receiver"
        if len(aggregate_data) == 0:
            aggregate_data = recv_data
        else:
            aggregate_data = pd.concat([aggregate_data, recv_data])
        # =------------------------------------------------------------= #

        alpha = 0.25

        # get color from available colors
        color = available_colors.pop(0)
        sender_ax.hist(
            sender_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        sender_ax.axvline(
            sender_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )
        receiver_ax.hist(
            recv_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        receiver_ax.axvline(
            recv_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )

    for direction in ["sender", "receiver"]:
        print()
        print(
            bcolors.OKGREEN
            + f"=-------------= {direction.capitalize()} | Latencies =-------------="
            + bcolors.ENDC
        )
        pre_sorted_data = aggregate_data[(aggregate_data["direction"] == direction)]
        # group by method, calculate mean of latency, then sort by mean latency
        pre_sorted_data = (
            pre_sorted_data.groupby("method")
            .agg({"latency": ["mean", "median", "std"]})
            .reset_index()
        )
        sorted_by_mean = pre_sorted_data.sort_values(by=[("latency", "mean")])
        for _, row in sorted_by_mean.iterrows():
            method = row["method"].iloc[0]
            mean_latency = format(row[("latency", "mean")] / 1000, ".2f")
            median_latency = format(row[("latency", "median")] / 1000, ".2f")
            std_latency = format(row[("latency", "std")] / 1000, ".2f")
            # format the lines to be more readable
            padding = " " * (20 - len(method))
            print(
                f"{method}{padding}| mu: {mean_latency}us | median: {median_latency}us | std: {std_latency}us"
            )

    sender_ax.legend()
    receiver_ax.legend()
    if show_graph:
        plt.show()


def analyze_intra_async_single_thread(
    bins, sender_threshold, receiver_threshold, show_graph=False
):
    available_colors = get_available_colors()
    methods = [
        "flume",
        "kanal",
        "tokio",
        "async_channel",
        "crossfire",
        "thingbuf",
        "tachyonix",
        "postage",
        "async_broadcast",
    ]  #

    aggregate_data = pd.DataFrame(columns=["latency", "method"])

    print()
    print(
        bcolors.OKCYAN
        + "=-= Intra-Process (Async Single Thread) | Latency Stats (ns) =-="
        + bcolors.ENDC
    )

    fig, sender_ax = plt.subplots(1)
    sender_ax.set_title(
        "Intra-Process (Async Single Thread) | Sender | Latency Distribution"
    )
    sender_ax.set_xlabel("Latency (ns)")
    sender_ax.set_ylabel("Density")

    fig, receiver_ax = plt.subplots(1)
    receiver_ax.set_title(
        "Intra-Process (Async Single Thread) | Receiver | Latency Distribution"
    )
    receiver_ax.set_xlabel("Latency (ns)")
    receiver_ax.set_ylabel("Density")

    for method in methods:
        # =------------------------------------------------------------= #
        with open(f"intra_latencies/async_single_thread/{method}_sender.txt", "r") as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        sender_data = pd.DataFrame(lines, columns=["latency"])
        sender_data = sender_data.astype(int)
        sender_data = sender_data[sender_data["latency"] < sender_threshold]
        sender_data["method"] = method
        sender_data["direction"] = "sender"
        if len(aggregate_data) == 0:
            aggregate_data = sender_data
        else:
            aggregate_data = pd.concat([aggregate_data, sender_data])
        # =------------------------------------------------------------= #
        with open(
            f"intra_latencies/async_single_thread/{method}_receiver.txt", "r"
        ) as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        recv_data = pd.DataFrame(lines, columns=["latency"])
        recv_data = recv_data.astype(int)
        recv_data = recv_data[recv_data["latency"] < receiver_threshold]
        recv_data["method"] = method
        recv_data["direction"] = "receiver"
        if len(aggregate_data) == 0:
            aggregate_data = recv_data
        else:
            aggregate_data = pd.concat([aggregate_data, recv_data])
        # =------------------------------------------------------------= #

        alpha = 0.25

        # get color from available colors
        color = available_colors.pop(0)
        sender_ax.hist(
            sender_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        sender_ax.axvline(
            sender_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )
        receiver_ax.hist(
            recv_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        receiver_ax.axvline(
            recv_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )

    for direction in ["sender", "receiver"]:
        print()
        print(
            bcolors.OKGREEN
            + f"=-------------= {direction.capitalize()} | Latencies =-------------="
            + bcolors.ENDC
        )
        pre_sorted_data = aggregate_data[(aggregate_data["direction"] == direction)]
        # group by method, calculate mean of latency, then sort by mean latency
        pre_sorted_data = (
            pre_sorted_data.groupby("method")
            .agg({"latency": ["mean", "median", "std"]})
            .reset_index()
        )
        sorted_by_mean = pre_sorted_data.sort_values(by=[("latency", "mean")])
        for _, row in sorted_by_mean.iterrows():
            method = row["method"].iloc[0]
            mean_latency = format(row[("latency", "mean")] / 1000, ".2f")
            median_latency = format(row[("latency", "median")] / 1000, ".2f")
            std_latency = format(row[("latency", "std")] / 1000, ".2f")
            # format the lines to be more readable
            padding = " " * (20 - len(method))
            print(
                f"{method}{padding}| mu: {mean_latency}us | median: {median_latency}us | std: {std_latency}us"
            )

    sender_ax.legend()
    receiver_ax.legend()
    if show_graph:
        plt.show()


def analyze_intra_async_multi_thread(
    bins, sender_threshold, receiver_threshold, show_graph=False
):

    available_colors = get_available_colors()
    methods = [
        "flume",
        "kanal",
        "tokio",
        "async_channel",
        "crossfire",
        "thingbuf",
        "tachyonix",
        "postage",
        "async_broadcast",
    ]  #

    aggregate_data = pd.DataFrame(columns=["latency", "method"])

    print()
    print(
        bcolors.OKCYAN
        + "=-= Intra-Process (Async Multi Thread) | Latency Stats (ns) =-="
        + bcolors.ENDC
    )

    fig, sender_ax = plt.subplots(1)
    sender_ax.set_title(
        "Intra-Process (Async Multi Thread) | Sender | Latency Distribution"
    )
    sender_ax.set_xlabel("Latency (ns)")
    sender_ax.set_ylabel("Density")

    fig, receiver_ax = plt.subplots(1)
    receiver_ax.set_title(
        "Intra-Process (Async Multi Thread) | Receiver | Latency Distribution"
    )
    receiver_ax.set_xlabel("Latency (ns)")
    receiver_ax.set_ylabel("Density")

    for method in methods:
        # =------------------------------------------------------------= #
        with open(f"intra_latencies/async_multi_thread/{method}_sender.txt", "r") as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        sender_data = pd.DataFrame(lines, columns=["latency"])
        sender_data = sender_data.astype(int)
        sender_data = sender_data[sender_data["latency"] < sender_threshold]
        sender_data["method"] = method
        sender_data["direction"] = "sender"
        if len(aggregate_data) == 0:
            aggregate_data = sender_data
        else:
            aggregate_data = pd.concat([aggregate_data, sender_data])
        # =------------------------------------------------------------= #
        with open(
            f"intra_latencies/async_multi_thread/{method}_receiver.txt", "r"
        ) as f:
            lines = f.readlines()
            lines = [x.rstrip() for x in lines]
            lines = [int(x) for x in lines if x != ""]
        recv_data = pd.DataFrame(lines, columns=["latency"])
        recv_data = recv_data.astype(int)
        recv_data = recv_data[recv_data["latency"] < receiver_threshold]
        recv_data["method"] = method
        recv_data["direction"] = "receiver"
        if len(aggregate_data) == 0:
            aggregate_data = recv_data
        else:
            aggregate_data = pd.concat([aggregate_data, recv_data])
        # =------------------------------------------------------------= #

        alpha = 0.25

        # get color from available colors
        color = available_colors.pop(0)
        sender_ax.hist(
            sender_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        sender_ax.axvline(
            sender_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )
        receiver_ax.hist(
            recv_data["latency"],
            bins=bins,
            alpha=alpha,
            density=True,
            color=color,
            label=method,
        )
        receiver_ax.axvline(
            recv_data["latency"].mean(), color=color, linestyle="solid", linewidth=1
        )

    for direction in ["sender", "receiver"]:
        print()
        print(
            bcolors.OKGREEN
            + f"=-------------= {direction.capitalize()} | Latencies =-------------="
            + bcolors.ENDC
        )
        pre_sorted_data = aggregate_data[(aggregate_data["direction"] == direction)]
        # group by method, calculate mean of latency, then sort by mean latency
        pre_sorted_data = (
            pre_sorted_data.groupby("method")
            .agg({"latency": ["mean", "median", "std"]})
            .reset_index()
        )
        sorted_by_mean = pre_sorted_data.sort_values(by=[("latency", "mean")])
        for _, row in sorted_by_mean.iterrows():
            method = row["method"].iloc[0]
            mean_latency = format(row[("latency", "mean")] / 1000, ".2f")
            median_latency = format(row[("latency", "median")] / 1000, ".2f")
            std_latency = format(row[("latency", "std")] / 1000, ".2f")
            # format the lines to be more readable
            padding = " " * (20 - len(method))
            print(
                f"{method}{padding}| mu: {mean_latency}us | median: {median_latency}us | std: {std_latency}us"
            )

    sender_ax.legend()
    receiver_ax.legend()
    if show_graph:
        plt.show()


# =------------------------------------------------------------= #
# Definitely a cleaner way to do this, but I'm in a rush for now

# Thresholds are used to cut off outliers in latency (if desired), there are denoted in nanoseconds (ns)
# 1000ns = 1us
# 1000us = 1ms

bins = 150

inter_async_sender_threshold = 15000
inter_async_receiver_threshold = 40000

inter_busy_sender_threshold = 7500
inter_busy_receiver_threshold = 15000

intra_async_single_thread_sender_threshold = 7500
intra_async_single_thread_receiver_threshold = 75000

intra_async_multi_thread_sender_threshold = 7500
intra_async_multi_thread_receiver_threshold = 60000

analyze_inter_async(bins, inter_async_sender_threshold, inter_async_receiver_threshold)
analyze_inter_busy(bins, inter_busy_sender_threshold, inter_busy_receiver_threshold)
analyze_intra_async_single_thread(
    bins,
    intra_async_single_thread_sender_threshold,
    intra_async_single_thread_receiver_threshold,
)
analyze_intra_async_multi_thread(
    bins,
    intra_async_multi_thread_sender_threshold,
    intra_async_multi_thread_receiver_threshold,
)
