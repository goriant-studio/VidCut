#include "MediaDecoder.h"

extern "C" {
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
}

namespace VidCut {

MediaDecoder::MediaDecoder(QObject* parent) : QObject(parent) {}

MediaDecoder::~MediaDecoder() {
    close();
}

bool MediaDecoder::open(const QString& filePath) {
    close();
    m_filePath = filePath;
    // TODO: avformat_open_input, find streams, open codec
    return false;
}

void MediaDecoder::close() {
    if (m_videoCtx) {
        avcodec_free_context(&m_videoCtx);
        m_videoCtx = nullptr;
    }
    if (m_formatCtx) {
        avformat_close_input(&m_formatCtx);
        m_formatCtx = nullptr;
    }
}

} // namespace VidCut
