/// 单链路由决策器 — Sprint 1
///
/// TODO Sprint 1 陆续补:
///   - Oracle quorum 3中2 gas price
///   - 评分公式 Speed×0.6 + Cost×0.3 + Safety×0.1
///   - Bridge 白名单过滤 (Stargate/CBridge)
///   - RPC dirty fuzz 注入测试
#[derive(Debug, Clone)]
pub struct Router {
    chains: Vec<Chain>,
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub id: u64,
    pub name: &'static str,
    pub gas: u64,
    pub score: f64,
    pub block_time_ms: u64, // ms
}

impl Router {
    pub fn new() -> Self {
        Router {
            chains: vec![
                Chain {
                    id: 1,
                    name: "ethereum",
                    gas: 25,
                    score: 0.55,
                    block_time_ms: 12000,
                },
                Chain {
                    id: 137,
                    name: "polygon",
                    gas: 3,
                    score: 0.91,
                    block_time_ms: 2000,
                },
                Chain {
                    id: 42161,
                    name: "arbitrum",
                    gas: 8,
                    score: 0.82,
                    block_time_ms: 250,
                },
                Chain {
                    id: 10,
                    name: "optimism",
                    gas: 6,
                    score: 0.87,
                    block_time_ms: 2000,
                },
            ],
        }
    }

    /// 选链: hint 精确匹配 > 最高分
    pub fn select(&self, hint: Option<&str>) -> Option<&Chain> {
        if let Some(h) = hint {
            if let Some(c) = self.chains.iter().find(|c| c.name == h) {
                return Some(c);
            }
        }
        self.chains.iter().max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Less)
        })
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
