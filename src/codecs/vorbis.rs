use super::{av::*, *};

codec_pair!(CodecId::Vorbis, None, AV_CODEC_ID_VORBIS, Vorbis);
parser_pair!(CodecId::Vorbis, AV_CODEC_ID_VORBIS, Vorbis);
