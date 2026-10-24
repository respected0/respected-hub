# common/

Üç depoyu (Hub, Brain, Orchestrator) birlikte ilgilendiren her şey burada durur. Ürünlerden biri üzerinde tek başına çalışırken bu klasöre gerek yoktur.
Araçlar üç deponun yan yana durduğunu varsayar ([ECOSYSTEM.md](ECOSYSTEM.md)).

| Klasör / dosya | Ne için |
|---|---|
| [ECOSYSTEM.md](ECOSYSTEM.md) | Ekosistem ve geliştirme ortamı: üç depo, klasör düzeni, kurulum |
| [release/](release/RELEASE.md) | Derleme, imzalama ve yayın (`tools/build`, `tools/signing`, `tools/acceptance`), sürüm notları, kendi testi |
| [conformance/](conformance/) | Ortak sözleşmenin (`docs/PRODUCT_CONTRACT.md`) üç ürüne karşı uyum testi |
| [brand/](brand/) | Üç ürünün logoları ve onları üreten `make_brand.py` |
| [measure/](measure/) | `measure-rpp.py`: kurulu ürünlere salt okunur RPP çağrılarının süresini ölçer |
