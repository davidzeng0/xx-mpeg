use super::{av::*, *};

codec_pair!(CodecId::Mp2, None, AV_CODEC_ID_MP2, Mp2);
parser_pair!(CodecId::Mp2, AV_CODEC_ID_MP2, Mp2);
