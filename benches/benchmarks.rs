#![allow(unused)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::{cmp::Reverse, collections::BinaryHeap, hint::black_box};

use feap::feap::*;

fn build_feap(size: u64) -> Feap<u64> {
    (0..size).fold(Feap::new(), |mut heap, i| {
        heap.insert(NodePtr::new(i));
        heap
    })
}
fn build_binary_heap(size: u64) -> BinaryHeap<Reverse<u64>> {
    (0..size).fold(BinaryHeap::new(), |mut heap, i| {
        heap.push(Reverse(i));
        heap
    })
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_insert");
    for &size in &[1, 10, 100, 1000, 10000, 100000, 1000000] {
        group.throughput(Throughput::Elements(size));
        group.bench_with_input(BenchmarkId::new("Feap", size), &size, |b, &size| {
            b.iter(|| {
                let mut heap = Feap::new();
                (0..size).for_each(|i| heap.insert(black_box(NodePtr::new(i))));
            });
        });
        group.bench_with_input(BenchmarkId::new("BinaryHeap", size), &size, |b, &size| {
            b.iter(|| {
                let mut heap = BinaryHeap::new();
                (0..size).for_each(|i| heap.push(black_box(i)));
            });
        });
    }
    group.finish();
}

fn bench_get_min(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_get_min");
    for &size in &[1, 10, 100, 1000, 10000, 100000, 1000000] {
        group.throughput(Throughput::Elements(size));
        group.bench_with_input(BenchmarkId::new("Feap", size), &size, |b, &size| {
            b.iter(|| {
                let mut feap = build_feap(size);
                for _ in 0..size {
                    let _ = feap.delete_min();
                }
            });
        });
        group.bench_with_input(BenchmarkId::new("BinaryHeap", size), &size, |b, &size| {
            b.iter(|| {
                let mut heap = build_binary_heap(size);
                for _ in 0..size {
                    let _ = heap.pop();
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_insert);
criterion_main!(benches);
