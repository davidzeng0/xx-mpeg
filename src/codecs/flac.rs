use super::{av::*, *};

codec_pair!(CodecId::Flac, None, AV_CODEC_ID_FLAC, Flac);
parser_pair!(CodecId::Flac, AV_CODEC_ID_FLAC, Flac);
