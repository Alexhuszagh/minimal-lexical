#!/usr/bin/env python
'''
    timings
    =======

    Plot the timings from building minimal-lexical.
'''

import argparse
import json
import subprocess
import os

import matplotlib.pyplot as plt
from matplotlib import patches
from matplotlib import textpath

plt.style.use('ggplot')

scripts = os.path.dirname(os.path.realpath(__file__))
home = os.path.dirname(scripts)

def parse_args(argv=None):
    '''Create and parse our command line arguments.'''

    parser = argparse.ArgumentParser(description='Time building minimal-lexical.')
    parser.add_argument(
        '--features',
        help='''optional features to add''',
        default='',
    )
    parser.add_argument(
        '--no-default-features',
        help='''disable default features''',
        action='store_true',
    )
    return parser.parse_args(argv)

def clean(directory=home):
    '''Clean the project'''

    os.chdir(directory)
    subprocess.check_call(
        ['cargo', '+nightly', 'clean'],
        shell=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

def build(args):
    '''Build the project and get the timings output.'''

    command = 'cargo +nightly build -Z timings=json'
    if args.no_default_features:
        command = f'{command} --no-default-features'
    if args.features:
        command = f'{command} --features={args.features}'
    process = subprocess.Popen(
        # Use shell for faster performance.
        # Spawning a new process is a **lot** slower, gives misleading info.
        command,
        shell=True,
        stderr=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
    )
    process.wait()
    data = {}
    for line in iter(process.stdout.readline, b''):
        line = line.decode('utf-8')
        crate = json.loads(line)
        name = crate['target']['name']
        data[name] = (crate['duration'], crate['rmeta_time'])

    process.stdout.close()

    return data

def filename(basename, args):
    '''Get a resilient name for the benchmark data.'''

    name = basename
    if args.no_default_features:
        name = f'{name}_nodefault'
    if args.features:
        name = f'{name}_features={args.features}'
    return name

def plot_timings(timings, output):
    '''Plot our timings data.'''

    offset = 0
    text_length = 0
    count = len(timings) + 1
    fig, ax = plt.subplots()
    bar_height = count * 0.05

    def plot_timing(name):
        '''Plot the timing of a specific value.'''

        nonlocal count
        nonlocal text_length

        if name not in timings:
            return
        duration, rmeta = timings[name]
        local_offset = offset
        ax.add_patch(patches.Rectangle(
            (offset, count - bar_height / 2), duration, bar_height,
            alpha=0.6,
            facecolor='lightskyblue',
            label=name,
        ))
        local_offset += rmeta
        ax.add_patch(patches.Rectangle(
            (local_offset, count - bar_height / 2), duration - rmeta, bar_height,
            alpha=0.6,
            facecolor='darkorchid',
            label=f'{name}_rmeta',
        ))
        local_offset += duration - rmeta
        text = f'minimal-lexical {round(duration, 2)}s'
        text_length = max(len(text), text_length)
        ax.annotate(
            text,
            xy=(local_offset + 0.02, count),
            xycoords='data',
            horizontalalignment='left',
            verticalalignment='center',
        )
        count -= 1

    def max_duration(*keys):
        '''Get the max duration from a list of keys.'''

        max_time = 0
        for key in keys:
            if key not in timings:
                continue
            max_time = max(timings[key][0], max_time)
        return max_time

    # Plot in order of our dependencies.
    plot_timing('minimal-lexical')
    offset += max_duration('minimal-lexical')

    title = 'Build Timings'
    ax.set_title(title)
    ax.set_xlabel('Time (s)')

    # Hide the y-axis labels.
    ax.set_yticks(list(range(1, len(timings) + 2)))
    ax.yaxis.set_tick_params(which='both', length=0)
    plt.setp(ax.get_yticklabels(), visible=False)

    # Ensure the canvas includes all the annotations.
    # 0.5 is long enough for the largest label.
    plt.xlim(0, offset + 0.02 * text_length)
    plt.ylim(count + 0.5, len(timings) + 1.5)

    # Save the figure.
    fig.savefig(output, format='svg')
    fig.clf()

def plot(args):
    '''Build and plot the timings for the root module.'''

    clean()
    timings = build(args)
    path = f'{home}/assets/timings_{filename("timings", args)}_{os.name}.svg'
    plot_timings(timings, path)

def main(argv=None):
    '''Entry point.'''

    args = parse_args(argv)
    plot(args)

if __name__ == '__main__':
    main()
